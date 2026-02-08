use git2::Repository;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::error::{ChabaError, Result};

pub struct GitOps {
    repo: Repository,
}

impl GitOps {
    /// Open repository from current directory or parent directories
    pub fn open() -> Result<Self> {
        let repo = Repository::discover(".").map_err(|_| ChabaError::NotInGitRepo)?;
        Ok(GitOps { repo })
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

        let output = Command::new("git")
            .current_dir(&repo_path)
            .args(["fetch", remote, branch])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::GitError(git2::Error::from_str(&error)));
        }

        Ok(())
    }

    /// Add a worktree
    pub async fn add_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let repo_path = self.repo_root();

        let output = Command::new("git")
            .current_dir(&repo_path)
            .args(["worktree", "add", path.to_str().unwrap(), branch])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::GitError(git2::Error::from_str(&error)));
        }

        Ok(())
    }

    /// Remove a worktree
    pub async fn remove_worktree(&self, path: &Path) -> Result<()> {
        let repo_path = self.repo_root();

        let output = Command::new("git")
            .current_dir(&repo_path)
            .args(["worktree", "remove", path.to_str().unwrap(), "--force"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::GitError(git2::Error::from_str(&error)));
        }

        Ok(())
    }

    /// List all worktrees
    pub async fn list_worktrees(&self) -> Result<Vec<PathBuf>> {
        let repo_path = self.repo_root();

        let output = Command::new("git")
            .current_dir(&repo_path)
            .args(["worktree", "list", "--porcelain"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ChabaError::GitError(git2::Error::from_str(&error)));
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
}

/// Get PR information using GitHub CLI
pub async fn get_pr_branch(pr_number: u32) -> Result<String> {
    // Check if gh is installed
    let gh_check = Command::new("which").arg("gh").output().await?;

    if !gh_check.status.success() {
        return Err(ChabaError::GhCliNotFound);
    }

    // Get PR branch name
    let output = Command::new("gh")
        .args(["pr", "view", &pr_number.to_string(), "--json", "headRefName", "-q", ".headRefName"])
        .output()
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
