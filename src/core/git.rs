use git2::Repository;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::core::command::{CommandRunner, LiveCommandRunner};
use crate::error::{ChabaError, Result};

/// Git statistics for a worktree
#[derive(Debug, Clone, Default)]
pub struct GitStats {
    /// Number of files changed
    pub files_changed: usize,
    /// Number of lines added
    pub lines_added: usize,
    /// Number of lines deleted
    pub lines_deleted: usize,
    /// Number of commits ahead of upstream
    pub commits_ahead: usize,
    /// Number of commits behind upstream
    pub commits_behind: usize,
    /// Current branch name
    pub current_branch: Option<String>,
    /// Upstream branch name (e.g., "origin/main")
    pub upstream_branch: Option<String>,
}

pub struct GitOps {
    repo: Repository,
    runner: Arc<dyn CommandRunner + Send + Sync>,
}

impl GitOps {
    /// Create a new GitOps instance with a specific repository and command runner
    ///
    /// This constructor is primarily for testing, allowing injection of a mock runner.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    /// * `runner` - Command runner implementation (LiveCommandRunner in production, mock in tests)
    pub fn new(repo_path: &Path, runner: Arc<dyn CommandRunner + Send + Sync>) -> Result<Self> {
        let repo = Repository::open(repo_path).map_err(|_| ChabaError::NotInGitRepo)?;
        Ok(GitOps { repo, runner })
    }

    /// Open repository from current directory or parent directories
    ///
    /// Uses the default LiveCommandRunner for production use.
    pub fn open() -> Result<Self> {
        let repo = Repository::discover(".").map_err(|_| ChabaError::NotInGitRepo)?;
        Ok(GitOps {
            repo,
            runner: Arc::new(LiveCommandRunner),
        })
    }

    /// Open repository from a specific path
    ///
    /// This is useful for testing where you want to specify the exact repository location.
    pub fn open_at(path: &Path) -> Result<Self> {
        Self::new(path, Arc::new(LiveCommandRunner))
    }

    /// Get repository root path
    pub fn repo_root(&self) -> PathBuf {
        self.repo
            .workdir()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }

    /// Fetch a branch from remote
    pub async fn fetch_branch(&self, remote: &str, branch: &str) -> Result<()> {
        let repo_path = self.repo_root();

        let output = self
            .runner
            .run(
                "git",
                &[
                    "fetch".as_ref(),
                    remote.as_ref(),
                    branch.as_ref(),
                ],
                &repo_path,
            )
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Git operation failed: {}",
                error
            )));
        }

        Ok(())
    }

    /// Add a worktree
    pub async fn add_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let repo_path = self.repo_root();

        let path_str = path
            .to_str()
            .ok_or_else(|| ChabaError::ConfigError(
                format!("Invalid path (non-UTF8): {}", path.display())
            ))?;

        let output = self
            .runner
            .run(
                "git",
                &[
                    "worktree".as_ref(),
                    "add".as_ref(),
                    OsStr::new(path_str),
                    branch.as_ref(),
                ],
                &repo_path,
            )
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Git operation failed: {}",
                error
            )));
        }

        Ok(())
    }

    /// Remove a worktree
    pub async fn remove_worktree(&self, path: &Path) -> Result<()> {
        let repo_path = self.repo_root();

        let path_str = path
            .to_str()
            .ok_or_else(|| ChabaError::ConfigError(
                format!("Invalid path (non-UTF8): {}", path.display())
            ))?;

        let output = self
            .runner
            .run(
                "git",
                &[
                    "worktree".as_ref(),
                    "remove".as_ref(),
                    OsStr::new(path_str),
                    "--force".as_ref(),
                ],
                &repo_path,
            )
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Git operation failed: {}",
                error
            )));
        }

        Ok(())
    }

    /// Get PR branch name using GitHub CLI
    pub async fn get_pr_branch(&self, pr_number: u32) -> Result<String> {
        let repo_path = self.repo_root();

        // Check if gh is installed
        let gh_check = self
            .runner
            .run("which", &["gh".as_ref()], &repo_path)
            .await?;

        if !gh_check.status.success() {
            return Err(ChabaError::GhCliNotFound);
        }

        // Get PR branch name
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "pr".as_ref(),
                    "view".as_ref(),
                    pr_number.to_string().as_ref(),
                    "--json".as_ref(),
                    "headRefName".as_ref(),
                    "-q".as_ref(),
                    ".headRefName".as_ref(),
                ],
                &repo_path,
            )
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("Could not resolve to a PullRequest") {
                return Err(ChabaError::PrNotFound(pr_number));
            }
            return Err(ChabaError::GhCliError(error.to_string()));
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if branch.is_empty() {
            return Err(ChabaError::PrNotFound(pr_number));
        }

        Ok(branch)
    }

    /// List all worktrees
    /// Reserved for Phase 3: AI Agent integration
    #[allow(dead_code)]
    pub async fn list_worktrees(&self) -> Result<Vec<PathBuf>> {
        let repo_path = self.repo_root();

        let output = self
            .runner
            .run(
                "git",
                &[
                    "worktree".as_ref(),
                    "list".as_ref(),
                    "--porcelain".as_ref(),
                ],
                &repo_path,
            )
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Git operation failed: {}",
                error
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();

        for line in stdout.lines() {
            if line.starts_with("worktree ") {
                let path = line.trim_start_matches("worktree ").trim();
                worktrees.push(PathBuf::from(path));
            }
        }

        Ok(worktrees)
    }

    /// Get git statistics for a worktree
    ///
    /// Returns information about file changes, commits ahead/behind, etc.
    pub async fn get_stats(&self, worktree_path: &Path) -> Result<GitStats> {
        let mut stats = GitStats::default();

        // Get current branch name
        let branch_output = self
            .runner
            .run(
                "git",
                &["rev-parse".as_ref(), "--abbrev-ref".as_ref(), "HEAD".as_ref()],
                worktree_path,
            )
            .await?;

        if branch_output.status.success() {
            stats.current_branch = Some(
                String::from_utf8_lossy(&branch_output.stdout)
                    .trim()
                    .to_string(),
            );
        }

        // Get upstream branch
        if let Some(ref branch) = stats.current_branch {
            let upstream_output = self
                .runner
                .run(
                    "git",
                    &[
                        "rev-parse".as_ref(),
                        "--abbrev-ref".as_ref(),
                        format!("{}@{{upstream}}", branch).as_ref(),
                    ],
                    worktree_path,
                )
                .await;

            if let Ok(output) = upstream_output {
                if output.status.success() {
                    stats.upstream_branch = Some(
                        String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    );
                }
            }
        }

        // Get diff stats (files changed, lines added/deleted)
        let diff_output = self
            .runner
            .run(
                "git",
                &["diff".as_ref(), "--stat".as_ref()],
                worktree_path,
            )
            .await?;

        if diff_output.status.success() {
            let diff_text = String::from_utf8_lossy(&diff_output.stdout);
            // Parse last line: "X files changed, Y insertions(+), Z deletions(-)"
            if let Some(summary_line) = diff_text.lines().last() {
                if let Some(files_part) = summary_line.split(',').next() {
                    if let Some(num_str) = files_part.split_whitespace().next() {
                        stats.files_changed = num_str.parse().unwrap_or(0);
                    }
                }

                for part in summary_line.split(',') {
                    if part.contains("insertion") {
                        if let Some(num_str) = part.split_whitespace().next() {
                            stats.lines_added = num_str.parse().unwrap_or(0);
                        }
                    } else if part.contains("deletion") {
                        if let Some(num_str) = part.split_whitespace().next() {
                            stats.lines_deleted = num_str.parse().unwrap_or(0);
                        }
                    }
                }
            }
        }

        // Get commits ahead/behind
        if let Some(ref upstream) = stats.upstream_branch {
            // Commits ahead
            let ahead_output = self
                .runner
                .run(
                    "git",
                    &[
                        "rev-list".as_ref(),
                        "--count".as_ref(),
                        format!("{}..HEAD", upstream).as_ref(),
                    ],
                    worktree_path,
                )
                .await?;

            if ahead_output.status.success() {
                let ahead_str = String::from_utf8_lossy(&ahead_output.stdout).trim().to_string();
                stats.commits_ahead = ahead_str.parse().unwrap_or(0);
            }

            // Commits behind
            let behind_output = self
                .runner
                .run(
                    "git",
                    &[
                        "rev-list".as_ref(),
                        "--count".as_ref(),
                        format!("HEAD..{}", upstream).as_ref(),
                    ],
                    worktree_path,
                )
                .await?;

            if behind_output.status.success() {
                let behind_str = String::from_utf8_lossy(&behind_output.stdout).trim().to_string();
                stats.commits_behind = behind_str.parse().unwrap_or(0);
            }
        }

        Ok(stats)
    }

    /// Check if worktree has uncommitted changes
    pub async fn has_uncommitted_changes(&self, worktree_path: &Path) -> Result<bool> {
        let status_output = self
            .runner
            .run(
                "git",
                &["status".as_ref(), "--porcelain".as_ref()],
                worktree_path,
            )
            .await?;

        Ok(!status_output.stdout.is_empty())
    }

    /// Merge a branch into the current branch in the worktree
    ///
    /// # Safety
    ///
    /// This operation:
    /// - Checks for uncommitted changes before merging
    /// - Detects merge conflicts
    /// - Returns detailed error messages
    pub async fn merge(&self, worktree_path: &Path, from_branch: &str) -> Result<()> {
        // Check for uncommitted changes
        if self.has_uncommitted_changes(worktree_path).await? {
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Cannot merge: worktree has uncommitted changes. Commit or stash them first."
            )));
        }

        // Perform the merge
        let merge_output = self
            .runner
            .run(
                "git",
                &["merge".as_ref(), from_branch.as_ref()],
                worktree_path,
            )
            .await?;

        if !merge_output.status.success() {
            let error = String::from_utf8_lossy(&merge_output.stderr);

            // Check for merge conflicts
            if error.contains("CONFLICT") || error.contains("Automatic merge failed") {
                return Err(ChabaError::Other(anyhow::anyhow!(
                    "Merge conflict detected. Resolve conflicts manually in the worktree:\n{}",
                    worktree_path.display()
                )));
            }

            return Err(ChabaError::Other(anyhow::anyhow!(
                "Merge failed: {}",
                error
            )));
        }

        Ok(())
    }

    /// Rebase the current branch onto another branch in the worktree
    ///
    /// # Safety
    ///
    /// This operation:
    /// - Checks for uncommitted changes before rebasing
    /// - Detects rebase conflicts
    /// - Returns detailed error messages
    pub async fn rebase(&self, worktree_path: &Path, onto_branch: &str) -> Result<()> {
        // Check for uncommitted changes
        if self.has_uncommitted_changes(worktree_path).await? {
            return Err(ChabaError::Other(anyhow::anyhow!(
                "Cannot rebase: worktree has uncommitted changes. Commit or stash them first."
            )));
        }

        // Perform the rebase
        let rebase_output = self
            .runner
            .run(
                "git",
                &["rebase".as_ref(), onto_branch.as_ref()],
                worktree_path,
            )
            .await?;

        if !rebase_output.status.success() {
            let error = String::from_utf8_lossy(&rebase_output.stderr);

            // Check for rebase conflicts
            if error.contains("CONFLICT") || error.contains("could not apply") {
                return Err(ChabaError::Other(anyhow::anyhow!(
                    "Rebase conflict detected. Resolve conflicts manually in the worktree:\n{}\nThen run: git rebase --continue",
                    worktree_path.display()
                )));
            }

            return Err(ChabaError::Other(anyhow::anyhow!(
                "Rebase failed: {}",
                error
            )));
        }

        Ok(())
    }
}

/// Deprecated: Use GitOps::get_pr_branch() instead
///
/// This function is kept for backward compatibility but will be removed in a future version.
#[deprecated(since = "0.1.0", note = "Use GitOps::get_pr_branch() instead")]
pub async fn get_pr_branch(pr_number: u32) -> Result<String> {
    let git_ops = GitOps::open()?;
    git_ops.get_pr_branch(pr_number).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::os::unix::process::ExitStatusExt; // For ExitStatus::from_raw
    use std::process::{ExitStatus, Output};
    use std::sync::Mutex;

    // Simple mock implementation for testing
    struct TestCommandRunner {
        calls: Mutex<Vec<Vec<String>>>,
        return_output: Output,
        return_outputs: Option<Vec<Output>>,
    }

    impl TestCommandRunner {
        fn new(output: Output) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                return_output: output,
                return_outputs: None,
            }
        }

        fn new_multi(outputs: Vec<Output>) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                return_output: outputs.first().unwrap().clone(),
                return_outputs: Some(outputs),
            }
        }

        fn get_calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl CommandRunner for TestCommandRunner {
        async fn run(
            &self,
            _program: &str,
            args: &[&OsStr],
            _current_dir: &Path,
        ) -> std::result::Result<Output, std::io::Error> {
            let mut calls = self.calls.lock().unwrap();
            calls.push(
                args.iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect(),
            );

            // If multiple outputs are provided, return based on call index
            if let Some(ref outputs) = self.return_outputs {
                let call_index = calls.len() - 1;
                if call_index < outputs.len() {
                    return Ok(outputs[call_index].clone());
                }
            }

            Ok(self.return_output.clone())
        }
    }

    // Helper to create a successful output
    fn success_output(stdout: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: vec![],
        }
    }

    // Helper to create a failed output
    fn error_output(stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[tokio::test]
    async fn test_fetch_branch_builds_correct_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo); // Close repo to avoid lock issues

        let mock_runner = Arc::new(TestCommandRunner::new(success_output("")));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner.clone()).unwrap();
        git_ops.fetch_branch("origin", "main").await.unwrap();

        let calls = mock_runner.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], vec!["fetch", "origin", "main"]);
    }

    #[tokio::test]
    async fn test_add_worktree_builds_correct_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        let mock_runner = Arc::new(TestCommandRunner::new(success_output("")));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner.clone()).unwrap();
        git_ops
            .add_worktree(&temp_dir.path().join("test-wt"), "feature")
            .await
            .unwrap();

        let calls = mock_runner.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0], "worktree");
        assert_eq!(calls[0][1], "add");
        assert!(calls[0][2].contains("test-wt"));
        assert_eq!(calls[0][3], "feature");
    }

    #[tokio::test]
    async fn test_remove_worktree_builds_correct_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        let mock_runner = Arc::new(TestCommandRunner::new(success_output("")));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner.clone()).unwrap();
        git_ops
            .remove_worktree(&temp_dir.path().join("test-wt"))
            .await
            .unwrap();

        let calls = mock_runner.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0], "worktree");
        assert_eq!(calls[0][1], "remove");
        assert!(calls[0][2].contains("test-wt"));
        assert_eq!(calls[0][3], "--force");
    }

    #[tokio::test]
    async fn test_fetch_branch_error_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        let mock_runner = Arc::new(TestCommandRunner::new(error_output("fatal: remote not found")));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner).unwrap();
        let result = git_ops.fetch_branch("origin", "main").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Git operation failed"));
    }

    #[tokio::test]
    async fn test_list_worktrees_parsing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        let porcelain_output = format!(
            "worktree {}\nHEAD abc123\nbranch refs/heads/main\n\nworktree {}/wt1\nHEAD def456\nbranch refs/heads/feature\n",
            temp_dir.path().display(),
            temp_dir.path().display()
        );

        let mock_runner = Arc::new(TestCommandRunner::new(success_output(&porcelain_output)));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner).unwrap();
        let worktrees = git_ops.list_worktrees().await.unwrap();

        assert_eq!(worktrees.len(), 2);
        assert!(worktrees[0].ends_with(temp_dir.path().file_name().unwrap()));
        assert!(worktrees[1].to_string_lossy().contains("wt1"));
    }

    #[tokio::test]
    async fn test_get_pr_branch_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        // Mock: both calls succeed
        let mock_runner = Arc::new(TestCommandRunner::new_multi(vec![
            success_output(""), // which gh succeeds
            success_output("feature/test-branch\n"), // gh pr view succeeds
        ]));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner.clone()).unwrap();
        let branch = git_ops.get_pr_branch(123).await.unwrap();

        assert_eq!(branch, "feature/test-branch");

        // Verify commands were called correctly
        let calls = mock_runner.get_calls();
        assert_eq!(calls.len(), 2); // which gh, gh pr view

        // Check gh pr view command
        assert_eq!(calls[1][0], "pr");
        assert_eq!(calls[1][1], "view");
        assert_eq!(calls[1][2], "123");
    }

    #[tokio::test]
    async fn test_get_pr_branch_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        // Mock: first call (which gh) succeeds, second call (gh pr view) fails
        let mock_runner = Arc::new(TestCommandRunner::new_multi(vec![
            success_output(""), // which gh succeeds
            error_output("Could not resolve to a PullRequest with the number of 999"),
        ]));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner).unwrap();
        let result = git_ops.get_pr_branch(999).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChabaError::PrNotFound(pr) => assert_eq!(pr, 999),
            e => panic!("Expected PrNotFound, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_pr_branch_gh_not_installed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        std::mem::drop(repo);

        // Mock 'which gh' failure
        let mock_runner = Arc::new(TestCommandRunner::new(error_output("gh: command not found")));

        let git_ops = GitOps::new(temp_dir.path(), mock_runner).unwrap();
        let result = git_ops.get_pr_branch(123).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChabaError::GhCliNotFound => (),
            e => panic!("Expected GhCliNotFound, got: {:?}", e),
        }
    }
}
