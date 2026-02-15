use chrono::Utc;
use std::path::{Path, PathBuf};
use path_clean::PathClean;

use crate::config::Config;
use crate::core::{git::GitOps, state::{ReviewState, State}};
use crate::error::{ChabaError, Result};

pub struct WorktreeManager {
    git: GitOps,
    config: Config,
}

impl WorktreeManager {
    pub fn new(config: Config) -> Result<Self> {
        let git = GitOps::open()?;
        Ok(WorktreeManager { git, config })
    }

    /// This function ensures that the resolved path is within the allowed `base_dir`.
    /// It works even if the paths do not exist on the filesystem.
    ///
    /// This function:
    /// 1. Handles both absolute and relative paths.
    /// 2. Normalizes the path, resolving `.` and `..` components.
    /// 3. Ensures the resulting path is inside the `base_dir`.
    fn validate_path_secure(path: &Path, base_dir: &Path) -> Result<PathBuf> {
        // Clean the base_dir to get a canonical representation without FS access
        let cleaned_base = base_dir.clean();

        // If the provided path is absolute, clean it.
        // If it's relative, join it to the base and then clean.
        let cleaned_path = if path.is_absolute() {
            path.clean()
        } else {
            cleaned_base.join(path).clean()
        };

        // Check if the cleaned path starts with the cleaned base directory.
        if cleaned_path.starts_with(&cleaned_base) {
            Ok(cleaned_path)
        } else {
            Err(ChabaError::ConfigError(format!(
                "Path traversal detected. Path '{}' is outside of base directory '{}'",
                path.display(),
                base_dir.display()
            )))
        }
    }


    /// Create a new worktree for the given PR or branch
    pub async fn create(&self, pr_number: Option<u32>, branch: Option<String>, force: bool, custom_path: Option<String>) -> Result<ReviewState> {
        // Determine branch name
        let (pr, branch_name) = match (pr_number, branch) {
            (Some(pr), None) => {
                let branch = self.git.get_pr_branch(pr).await?;
                (pr, branch)
            }
            (None, Some(branch)) => {
                // Generate PR number from branch name hash (for tracking)
                let pr = Self::hash_branch_name(&branch);
                (pr, branch)
            }
            _ => return Err(ChabaError::InvalidInput),
        };

        // Determine and validate worktree path
        let worktree_path = if let Some(custom) = custom_path {
            let path = PathBuf::from(custom);
            Self::validate_path_secure(&path, &self.config.worktree.base_dir)?
        } else {
            let name = self.config.worktree.naming_template.replace("{pr}", &pr.to_string());
            let path = self.config.worktree.base_dir.join(name);
            // Validate the auto-generated path to ensure it's clean and within the base dir.
            Self::validate_path_secure(&path, &self.config.worktree.base_dir)?
        };

        // Check if worktree already exists
        if worktree_path.exists() {
            if force {
                // Force flag: remove without asking
                self.git.remove_worktree(&worktree_path).await?;
                tokio::fs::remove_dir_all(&worktree_path).await?;
            } else {
                // Interactive mode: ask user if they want to overwrite
                use dialoguer::Confirm;

                let overwrite = Confirm::new()
                    .with_prompt(format!(
                        "Worktree already exists at {}. Overwrite?",
                        worktree_path.display()
                    ))
                    .default(false)
                    .interact()
                    .unwrap_or(false);

                if overwrite {
                    self.git.remove_worktree(&worktree_path).await?;
                    tokio::fs::remove_dir_all(&worktree_path).await?;
                } else {
                    return Err(ChabaError::WorktreeExists(worktree_path));
                }
            }
        }

        // Create base directory if it doesn't exist
        if let Some(parent) = worktree_path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        // Fetch the branch
        tracing::info!("Fetching branch: {}", branch_name);
        self.git.fetch_branch("origin", &branch_name).await?;

        // Create worktree
        tracing::info!("Creating worktree at: {}", worktree_path.display());
        self.git.add_worktree(&worktree_path, &format!("origin/{}", branch_name)).await?;

        // Phase 2: Setup sandbox environment
        let mut state = State::load()?;
        let sandbox_manager = super::sandbox::SandboxManager::new(self.config.sandbox.clone());
        let sandbox_info = sandbox_manager
            .setup(&worktree_path, &self.git.repo_root(), &state)
            .await?;

        // Create review state with sandbox info
        let review = ReviewState {
            pr_number: pr,
            branch: branch_name.clone(),
            worktree_path: worktree_path.clone(),
            created_at: Utc::now(),
            port: sandbox_info.port,
            project_type: sandbox_info.project_type,
            deps_installed: sandbox_info.deps_installed,
            env_copied: sandbox_info.env_copied,
            agent_analyses: Vec::new(),
        };

        // Save state
        state.add_review(review.clone())?;

        Ok(review)
    }

    /// Remove a worktree
    pub async fn remove(&self, pr_number: u32) -> Result<()> {
        let mut state = State::load()?;

        let review = state
            .get_review(pr_number)
            .ok_or(ChabaError::WorktreeNotFound(pr_number))?
            .clone();

        // Remove worktree
        tracing::info!("Removing worktree at: {}", review.worktree_path.display());
        self.git.remove_worktree(&review.worktree_path).await?;

        // Remove from state
        state.remove_review(pr_number)?;

        Ok(())
    }

    /// List all active worktrees
    pub fn list(&self) -> Result<Vec<ReviewState>> {
        let state = State::load()?;
        Ok(state.reviews)
    }

    /// Generate a pseudo-PR number from branch name for non-PR branches
    fn hash_branch_name(branch: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        branch.hash(&mut hasher);
        (hasher.finish() % 10000) as u32 + 90000 // Range: 90000-99999
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_branch_name_range() {
        let pr = WorktreeManager::hash_branch_name("feature/test");
        assert!(pr >= 90000 && pr < 100000, "Hash should be in range 90000-99999");
    }

    #[test]
    fn test_hash_branch_name_deterministic() {
        let pr1 = WorktreeManager::hash_branch_name("feature/test");
        let pr2 = WorktreeManager::hash_branch_name("feature/test");
        assert_eq!(pr1, pr2, "Hash should be deterministic");
    }

    #[test]
    fn test_hash_branch_name_different() {
        let pr1 = WorktreeManager::hash_branch_name("feature/test1");
        let pr2 = WorktreeManager::hash_branch_name("feature/test2");
        assert_ne!(pr1, pr2, "Different branches should have different hashes");
    }

    // Property-based tests
    mod proptest_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_hash_always_in_range(branch in "[a-zA-Z0-9/_-]{1,100}") {
                let pr = WorktreeManager::hash_branch_name(&branch);
                prop_assert!(pr >= 90000 && pr < 100000);
            }

            #[test]
            fn test_hash_is_deterministic(branch in "[a-zA-Z0-9/_-]{1,100}") {
                let pr1 = WorktreeManager::hash_branch_name(&branch);
                let pr2 = WorktreeManager::hash_branch_name(&branch);
                prop_assert_eq!(pr1, pr2);
            }

            #[test]
            fn test_empty_string_does_not_panic(branch in ".*") {
                // This test just ensures no panic occurs with any string
                let pr = WorktreeManager::hash_branch_name(&branch);
                // Allow any value - we're just testing for panics
                prop_assert!(pr >= 90000);
            }
        }
    }
}
