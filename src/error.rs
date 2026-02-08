use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChabaError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("GitHub CLI not found. Please install it: brew install gh")]
    GhCliNotFound,

    #[error("GitHub CLI command failed: {0}")]
    GhCliError(String),

    #[error("Pull request #{0} not found")]
    PrNotFound(u32),

    #[error("Worktree already exists at {0}. Use --force to overwrite.")]
    WorktreeExists(PathBuf),

    #[error("Worktree not found for PR #{0}")]
    WorktreeNotFound(u32),

    #[error("Not in a git repository. Please run this command from within a git repository.")]
    NotInGitRepo,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_yaml::Error),

    #[error("Invalid PR number or branch name")]
    InvalidInput,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ChabaError>;
