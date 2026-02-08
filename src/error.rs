//! Error types for Chaba.
//!
//! This module defines all error types that can occur during Chaba operations.
//! All errors implement the `std::error::Error` trait via `thiserror`.
//!
//! # Examples
//!
//! ```rust
//! use chaba::error::{ChabaError, Result};
//!
//! fn example() -> Result<()> {
//!     // Operations that might fail
//!     Err(ChabaError::NotInGitRepo)
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Get platform-specific installation instructions for GitHub CLI
fn get_gh_install_instructions() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "  macOS: brew install gh\n  or visit: https://cli.github.com"
    }
    #[cfg(target_os = "linux")]
    {
        "  Ubuntu/Debian: sudo apt install gh\n  Fedora: sudo dnf install gh\n  or visit: https://cli.github.com"
    }
    #[cfg(target_os = "windows")]
    {
        "  Windows: winget install --id GitHub.cli\n  or: choco install gh\n  or visit: https://cli.github.com"
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        "  Visit: https://cli.github.com"
    }
}

/// Error types for Chaba operations.
///
/// This enum covers all possible errors that can occur during:
/// - Git operations (worktree creation, branch fetching)
/// - GitHub CLI operations (PR information retrieval)
/// - Configuration loading and parsing
/// - State management
/// - AI agent execution
///
/// # Examples
///
/// ```rust
/// use chaba::error::ChabaError;
///
/// let err = ChabaError::PrNotFound(123);
/// assert_eq!(err.to_string(), "Pull request #123 not found");
/// ```
#[derive(Error, Debug)]
pub enum ChabaError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("GitHub CLI not found. Please install it:\n{}", get_gh_install_instructions())]
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

    #[error("No available port in range {range_start}-{range_end}. Try cleaning up old review environments.")]
    NoAvailablePort { range_start: u16, range_end: u16 },

    #[error("AI agent '{agent}' execution failed\nstderr: {stderr}")]
    AgentExecutionError {
        agent: String,
        stdout: String,
        stderr: String,
    },

    #[error("State file was modified by another process. Expected version {expected}, but found {actual}. Please reload and try again.")]
    StateConflict { expected: u64, actual: u64 },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ChabaError>;
