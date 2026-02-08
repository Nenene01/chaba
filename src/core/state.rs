use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    pub pr_number: u32,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub reviews: Vec<ReviewState>,
}

impl State {
    /// Load state from file
    pub fn load() -> Result<Self> {
        let state_path = Self::state_file_path()?;

        if !state_path.exists() {
            return Ok(State::default());
        }

        let content = std::fs::read_to_string(&state_path)?;
        let state: State = serde_yaml::from_str(&content)?;
        Ok(state)
    }

    /// Save state to file
    pub fn save(&self) -> Result<()> {
        let state_path = Self::state_file_path()?;

        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(&self)?;
        std::fs::write(&state_path, content)?;
        Ok(())
    }

    /// Add a review to state
    pub fn add_review(&mut self, review: ReviewState) -> Result<()> {
        // Remove existing review with same PR number
        self.reviews.retain(|r| r.pr_number != review.pr_number);
        self.reviews.push(review);
        self.save()
    }

    /// Remove a review from state
    pub fn remove_review(&mut self, pr_number: u32) -> Result<()> {
        self.reviews.retain(|r| r.pr_number != pr_number);
        self.save()
    }

    /// Get review by PR number
    pub fn get_review(&self, pr_number: u32) -> Option<&ReviewState> {
        self.reviews.iter().find(|r| r.pr_number == pr_number)
    }

    /// Get state file path
    fn state_file_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::ChabaError::ConfigError("Cannot find home directory".to_string())
        })?;

        Ok(home.join(".chaba").join("state.yaml"))
    }
}
