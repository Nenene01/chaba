use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::HooksConfig;

/// Hook execution manager
pub struct HookManager {
    config: HooksConfig,
}

impl HookManager {
    /// Create a new HookManager
    pub fn new(config: HooksConfig) -> Self {
        HookManager { config }
    }

    /// Run post-create hook asynchronously
    ///
    /// This function spawns a background task to run the hook.
    /// The hook execution does not block the worktree creation.
    ///
    /// # Environment Variables
    ///
    /// The following environment variables are set for the hook:
    /// - `CHABA_WORKTREE_PATH`: Absolute path to the worktree
    /// - `CHABA_BRANCH`: Branch name
    /// - `CHABA_PR`: PR number
    pub fn run_post_create(
        &self,
        worktree_path: &Path,
        branch: &str,
        pr_number: u32,
    ) {
        let Some(hook_command) = &self.config.post_create else {
            // No hook configured
            return;
        };

        let command = hook_command.clone();
        let path = worktree_path.to_path_buf();
        let branch_name = branch.to_string();

        // Spawn async task to run hook in background
        tokio::spawn(async move {
            tracing::info!("Running post-create hook in background");

            let result = Command::new("sh")
                .arg("-c")
                .arg(&command)
                .env("CHABA_WORKTREE_PATH", &path)
                .env("CHABA_BRANCH", &branch_name)
                .env("CHABA_PR", pr_number.to_string())
                .current_dir(&path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await;

            match result {
                Ok(output) => {
                    if output.status.success() {
                        tracing::info!("Post-create hook completed successfully");
                        if !output.stdout.is_empty() {
                            tracing::debug!(
                                "Hook stdout: {}",
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                    } else {
                        tracing::warn!(
                            "Post-create hook failed with status: {}",
                            output.status
                        );
                        if !output.stderr.is_empty() {
                            tracing::warn!(
                                "Hook stderr: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute post-create hook: {}", e);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_hook_manager_no_hook() {
        let config = HooksConfig {
            post_create: None,
        };
        let manager = HookManager::new(config);

        // Should not panic when no hook is configured
        manager.run_post_create(&PathBuf::from("/tmp"), "test-branch", 123);
    }

    #[tokio::test]
    async fn test_hook_manager_with_simple_command() {
        let config = HooksConfig {
            post_create: Some("echo 'Hello from hook'".to_string()),
        };
        let manager = HookManager::new(config);

        manager.run_post_create(&PathBuf::from("/tmp"), "test-branch", 123);

        // Give the background task some time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
