//! Integration tests for git operations
//!
//! These tests create real git repositories in temporary directories
//! to verify the actual behavior of GitOps methods.

use chaba::core::git::GitOps;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;

/// Helper function to run git commands in a directory
async fn run_git(dir: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {:?} failed: {}", args, stderr).into());
    }

    Ok(())
}

/// Helper to set up a test git repository with initial commit
async fn setup_test_repo(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize repository
    run_git(dir, &["init"]).await?;

    // Configure git user
    run_git(dir, &["config", "user.email", "test@example.com"]).await?;
    run_git(dir, &["config", "user.name", "Test User"]).await?;

    // Create initial commit
    fs::write(dir.join("README.md"), "# Test Repository\n")?;
    run_git(dir, &["add", "."]).await?;
    run_git(dir, &["commit", "-m", "Initial commit"]).await?;

    Ok(())
}

#[tokio::test]
async fn test_gitops_open_at_valid_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    setup_test_repo(temp_dir.path()).await?;

    // Should successfully open the repository
    let git_ops = GitOps::open_at(temp_dir.path())?;
    let root = git_ops.repo_root();

    // Use canonicalize to handle symlink differences on macOS (/var vs /private/var)
    assert_eq!(root.canonicalize()?, temp_dir.path().canonicalize()?);

    Ok(())
}

#[tokio::test]
async fn test_gitops_open_at_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize as git repo

    // Should fail to open
    let result = GitOps::open_at(temp_dir.path());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_and_remove_worktree() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    setup_test_repo(temp_dir.path()).await?;

    // Create a branch
    run_git(temp_dir.path(), &["branch", "feature-branch"]).await?;

    let git_ops = GitOps::open_at(temp_dir.path())?;
    let worktree_path = temp_dir.path().join("worktree-test");

    // Add worktree
    git_ops.add_worktree(&worktree_path, "feature-branch").await?;

    // Verify worktree exists
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Remove worktree
    git_ops.remove_worktree(&worktree_path).await?;

    // Verify worktree is removed
    assert!(!worktree_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_fetch_branch_from_local_remote() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: Create a bare repository as "origin"
    let origin_dir = TempDir::new()?;
    run_git(origin_dir.path(), &["init", "--bare"]).await?;

    // Setup: Create a working repository and push to origin
    let work_dir = TempDir::new()?;
    setup_test_repo(work_dir.path()).await?;

    // Add origin remote
    let origin_url = origin_dir.path().to_str().unwrap();
    run_git(work_dir.path(), &["remote", "add", "origin", origin_url]).await?;

    // Push to origin
    run_git(work_dir.path(), &["push", "-u", "origin", "master"]).await?;

    // Create a new clone to test fetch
    let clone_parent = TempDir::new()?;
    let clone_path = clone_parent.path().join("clone");
    run_git(
        clone_parent.path(),
        &["clone", origin_url, clone_path.to_str().unwrap()],
    )
    .await?;

    // Test fetch_branch
    let git_ops = GitOps::open_at(&clone_path)?;
    git_ops.fetch_branch("origin", "master").await?;

    // If fetch succeeds without error, the test passes
    Ok(())
}

#[tokio::test]
async fn test_list_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    setup_test_repo(temp_dir.path()).await?;

    // Create branches
    run_git(temp_dir.path(), &["branch", "branch1"]).await?;
    run_git(temp_dir.path(), &["branch", "branch2"]).await?;

    let git_ops = GitOps::open_at(temp_dir.path())?;

    // Add worktrees
    let wt1 = temp_dir.path().join("wt1");
    let wt2 = temp_dir.path().join("wt2");
    git_ops.add_worktree(&wt1, "branch1").await?;
    git_ops.add_worktree(&wt2, "branch2").await?;

    // List worktrees
    let worktrees = git_ops.list_worktrees().await?;

    // Should have main worktree + 2 additional worktrees
    assert!(worktrees.len() >= 3);
    assert!(worktrees.iter().any(|p| p.ends_with("wt1")));
    assert!(worktrees.iter().any(|p| p.ends_with("wt2")));

    Ok(())
}
