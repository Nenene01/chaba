use chrono::Utc;
use std::path::PathBuf;

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

    /// Create a new worktree for the given PR or branch
    pub async fn create(&self, pr_number: Option<u32>, branch: Option<String>, force: bool, custom_path: Option<String>) -> Result<ReviewState> {
        // Determine branch name
        let (pr, branch_name) = match (pr_number, branch) {
            (Some(pr), None) => {
                let branch = super::git::get_pr_branch(pr).await?;
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
            PathBuf::from(custom)
        } else {
            let name = self.config.worktree.naming_template.replace("{pr}", &pr.to_string());
            self.config.worktree.base_dir.join(name)
        };

        // Check if worktree already exists
        if worktree_path.exists() && !force {
            return Err(ChabaError::WorktreeExists(worktree_path));
        }

        // Remove existing worktree if force is enabled
        if worktree_path.exists() && force {
            self.git.remove_worktree(&worktree_path).await?;
            tokio::fs::remove_dir_all(&worktree_path).await?;
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

        // Create review state
        let review = ReviewState {
            pr_number: pr,
            branch: branch_name.clone(),
            worktree_path: worktree_path.clone(),
            created_at: Utc::now(),
        };

        // Save state
        let mut state = State::load()?;
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
