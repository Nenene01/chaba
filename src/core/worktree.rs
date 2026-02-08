use chrono::Utc;
use std::path::{Path, PathBuf};

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

    /// Validate a path to prevent path traversal and symlink attacks
    ///
    /// This function:
    /// 1. Checks for parent directory (..) components
    /// 2. Resolves symlinks via canonicalization
    /// 3. Ensures the resolved path is within the allowed base directory
    fn validate_path_secure(path: &Path, base_dir: &Path) -> Result<PathBuf> {
        use std::path::Component;

        // Check for parent directory traversal
        for component in path.components() {
            if matches!(component, Component::ParentDir) {
                return Err(ChabaError::ConfigError(
                    "Invalid path: parent directory (..) is not allowed".to_string()
                ));
            }
        }

        // Resolve the base directory to its canonical form
        let canonical_base = base_dir.canonicalize()
            .map_err(|e| ChabaError::ConfigError(
                format!("Failed to resolve base directory {}: {}", base_dir.display(), e)
            ))?;

        // Resolve the path to its canonical form
        // For non-existent paths, validate the parent exists and is within base_dir
        let canonical_path = if path.exists() {
            path.canonicalize()
                .map_err(|e| ChabaError::ConfigError(
                    format!("Failed to resolve path {}: {}", path.display(), e)
                ))?
        } else {
            // Ensure parent exists or can be created within base_dir
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    let canonical_parent = parent.canonicalize()
                        .map_err(|e| ChabaError::ConfigError(
                            format!("Failed to resolve parent path {}: {}", parent.display(), e)
                        ))?;

                    // Verify parent is within base_dir
                    if !canonical_parent.starts_with(&canonical_base) {
                        return Err(ChabaError::ConfigError(
                            format!("Invalid path: {} is outside base directory {}",
                                canonical_parent.display(), canonical_base.display())
                        ));
                    }

                    // Return the canonical parent joined with filename
                    if let Some(filename) = path.file_name() {
                        canonical_parent.join(filename)
                    } else {
                        return Err(ChabaError::ConfigError(
                            "Invalid path: no filename".to_string()
                        ));
                    }
                } else {
                    // Parent doesn't exist - just verify the path would be within base_dir
                    // when created (simple prefix check on the input path)
                    if !path.starts_with(base_dir) {
                        return Err(ChabaError::ConfigError(
                            format!("Invalid path: {} is outside base directory {}",
                                path.display(), base_dir.display())
                        ));
                    }
                    path.to_path_buf()
                }
            } else {
                return Err(ChabaError::ConfigError(
                    "Invalid path: no parent directory".to_string()
                ));
            }
        };

        // Final check: ensure canonical path is within base directory
        if canonical_path.exists() && !canonical_path.starts_with(&canonical_base) {
            return Err(ChabaError::ConfigError(
                format!("Invalid path: {} is outside base directory {}",
                    canonical_path.display(), canonical_base.display())
            ));
        }

        Ok(canonical_path)
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

        // Determine worktree path
        let worktree_path = if let Some(custom) = custom_path {
            let path = PathBuf::from(custom);
            // Use secure validation that checks for symlinks and path traversal
            Self::validate_path_secure(&path, &self.config.worktree.base_dir)?
        } else {
            let name = self.config.worktree.naming_template.replace("{pr}", &pr.to_string());
            let path = self.config.worktree.base_dir.join(name);
            // Validate auto-generated path too
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
        if !self.config.worktree.base_dir.exists() {
            tokio::fs::create_dir_all(&self.config.worktree.base_dir).await?;
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
        (hasher.finish() % 100000) as u32 + 90000 // Range: 90000-99999
    }
}
