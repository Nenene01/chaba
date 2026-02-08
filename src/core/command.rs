//! Command execution abstraction for testability
//!
//! This module provides a trait-based abstraction over `tokio::process::Command`
//! to enable mocking in tests while using real command execution in production.

use async_trait::async_trait;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Output;

/// Trait for executing external commands
///
/// This abstraction allows for dependency injection of command execution logic,
/// making it possible to mock command execution in tests while using real
/// `tokio::process::Command` in production.
///
/// # Examples
///
/// ```rust
/// use chaba::core::command::{CommandRunner, LiveCommandRunner};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runner: Arc<dyn CommandRunner + Send + Sync> = Arc::new(LiveCommandRunner);
/// let output = runner.run(
///     "echo",
///     &["hello".as_ref()],
///     std::env::current_dir()?.as_path()
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait CommandRunner {
    /// Execute a command with the given arguments in the specified directory
    ///
    /// # Arguments
    ///
    /// * `program` - The program to execute (e.g., "git", "gh")
    /// * `args` - Command arguments
    /// * `current_dir` - Working directory for command execution
    ///
    /// # Returns
    ///
    /// Returns the command output including status, stdout, and stderr
    async fn run(
        &self,
        program: &str,
        args: &[&OsStr],
        current_dir: &Path,
    ) -> Result<Output, std::io::Error>;
}

/// Production implementation using tokio::process::Command
///
/// This is the default implementation used in production code.
/// It directly executes commands using `tokio::process::Command`.
pub struct LiveCommandRunner;

#[async_trait]
impl CommandRunner for LiveCommandRunner {
    async fn run(
        &self,
        program: &str,
        args: &[&OsStr],
        current_dir: &Path,
    ) -> Result<Output, std::io::Error> {
        tokio::process::Command::new(program)
            .current_dir(current_dir)
            .args(args)
            .output()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_live_runner_executes_command() {
        let runner = LiveCommandRunner;
        let output = runner
            .run(
                "echo",
                &["test".as_ref()],
                std::env::current_dir().unwrap().as_path(),
            )
            .await
            .unwrap();

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("test"));
    }
}
