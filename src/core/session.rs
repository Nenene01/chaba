use std::path::{Path, PathBuf};
use crate::error::{ChabaError, Result};

/// Session data manager for Claude Code
pub struct SessionManager {
    claude_dir: PathBuf,
}

impl SessionManager {
    /// Create a new SessionManager
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| ChabaError::ConfigError("Cannot find home directory".to_string()))?;
        let claude_dir = home.join(".claude").join("projects");

        Ok(SessionManager { claude_dir })
    }

    /// Convert a filesystem path to Claude's escaped format
    /// Example: /Users/foo/bar -> -Users-foo-bar
    fn escape_path(path: &Path) -> String {
        path.to_string_lossy()
            .replace('/', "-")
    }

    /// Get the session directory path for a given worktree path
    fn session_dir_for_path(&self, worktree_path: &Path) -> PathBuf {
        let escaped = Self::escape_path(worktree_path);
        self.claude_dir.join(escaped)
    }

    /// Copy session data from source worktree to target worktree
    ///
    /// This copies all Claude Code session data including:
    /// - sessions-index.json
    /// - All .jsonl session files
    ///
    /// Returns Ok(true) if copy succeeded, Ok(false) if source doesn't exist
    /// Returns Err only for critical failures
    pub async fn copy_session_data(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> Result<bool> {
        let source_session_dir = self.session_dir_for_path(source_path);
        let target_session_dir = self.session_dir_for_path(target_path);

        // Check if source session directory exists
        if !source_session_dir.exists() {
            tracing::info!(
                "Source session directory does not exist: {}",
                source_session_dir.display()
            );
            return Ok(false);
        }

        // Create target session directory
        tokio::fs::create_dir_all(&target_session_dir).await?;

        // Copy sessions-index.json if it exists
        let source_index = source_session_dir.join("sessions-index.json");
        if source_index.exists() {
            let target_index = target_session_dir.join("sessions-index.json");

            match tokio::fs::copy(&source_index, &target_index).await {
                Ok(_) => {
                    tracing::info!("Copied sessions-index.json");
                }
                Err(e) => {
                    tracing::warn!("Failed to copy sessions-index.json: {}", e);
                    // Continue anyway
                }
            }
        }

        // Copy all .jsonl files
        let mut dir_entries = tokio::fs::read_dir(&source_session_dir).await?;
        let mut copied_count = 0;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Some(filename) = path.file_name() {
                    let target_file = target_session_dir.join(filename);

                    match tokio::fs::copy(&path, &target_file).await {
                        Ok(_) => {
                            copied_count += 1;
                            tracing::debug!("Copied session file: {:?}", filename);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to copy {:?}: {}", filename, e);
                            // Continue with other files
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Copied {} session file(s) from {} to {}",
            copied_count,
            source_session_dir.display(),
            target_session_dir.display()
        );

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_path() {
        let path = Path::new("/Users/foo/bar");
        assert_eq!(SessionManager::escape_path(path), "-Users-foo-bar");

        let path = Path::new("/home/user/project");
        assert_eq!(SessionManager::escape_path(path), "-home-user-project");
    }

    #[test]
    fn test_escape_path_no_leading_slash() {
        let path = Path::new("relative/path");
        assert_eq!(SessionManager::escape_path(path), "relative-path");
    }
}
