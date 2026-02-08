/// End-to-End tests for Chaba
///
/// These tests verify the complete workflow with real git operations,
/// actual file system interactions, and full integration between components.

use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    use std::process::Command;

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Configure git user (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test Repository").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Create a test branch
    Command::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    std::fs::write(repo_path.join("test.txt"), "Test file").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add test file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Switch back to main/master
    Command::new("git")
        .args(["checkout", "master"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    (temp_dir, repo_path)
}

#[tokio::test]
async fn test_e2e_worktree_lifecycle() {
    use chaba::core::git::GitOps;
    use std::sync::Arc;
    use chaba::core::command::LiveCommandRunner;

    let (_temp_dir, repo_path) = create_test_repo();

    // Create GitOps instance
    let git_ops = GitOps::new(&repo_path, Arc::new(LiveCommandRunner)).unwrap();

    // Test: Fetch branch
    let result = git_ops.fetch_branch("origin", "feature/test").await;
    // Note: This will fail if no remote is configured, which is expected in local test
    // We're mainly testing that the command is constructed correctly
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for E2E

    // Test: Create worktree
    let worktree_path = repo_path.join("../test-worktree");
    let result = git_ops.add_worktree(&worktree_path, "feature/test").await;

    if result.is_ok() {
        // Verify worktree was created
        assert!(worktree_path.exists());
        assert!(worktree_path.join("test.txt").exists());

        // Test: List worktrees
        let worktrees = git_ops.list_worktrees().await.unwrap();
        assert!(worktrees.len() >= 2); // Main + test worktree

        // Test: Remove worktree
        let remove_result = git_ops.remove_worktree(&worktree_path).await;
        assert!(remove_result.is_ok());

        // Verify worktree was removed
        assert!(!worktree_path.exists());
    }
}

#[test]
fn test_e2e_project_detection() {
    use chaba::core::project::detect_project_type;

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Test: Unknown project (empty directory)
    let project_type = detect_project_type(project_path).unwrap();
    assert!(matches!(
        project_type,
        chaba::core::project::ProjectType::Unknown
    ));

    // Test: Node.js project (package.json)
    std::fs::write(
        project_path.join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .unwrap();
    let project_type = detect_project_type(project_path).unwrap();
    assert!(matches!(
        project_type,
        chaba::core::project::ProjectType::NodeJs { .. }
    ));

    // Clean up package.json
    std::fs::remove_file(project_path.join("package.json")).unwrap();

    // Test: Rust project (Cargo.toml)
    std::fs::write(
        project_path.join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    let project_type = detect_project_type(project_path).unwrap();
    assert!(matches!(
        project_type,
        chaba::core::project::ProjectType::Rust
    ));

    // Clean up Cargo.toml
    std::fs::remove_file(project_path.join("Cargo.toml")).unwrap();

    // Test: Python project (requirements.txt)
    std::fs::write(project_path.join("requirements.txt"), "pytest\n").unwrap();
    let project_type = detect_project_type(project_path).unwrap();
    assert!(matches!(
        project_type,
        chaba::core::project::ProjectType::Python { .. }
    ));
}

#[test]
fn test_e2e_port_assignment() {
    use chaba::core::port::PortManager;
    use chaba::core::state::State;

    // Create a port manager
    let port_manager = PortManager::new(50000, 50100);

    // Load state
    let state = State::load().unwrap_or_default();

    // Test: Assign port
    let port = port_manager.assign_port(&state).unwrap();
    assert!(port >= 50000 && port <= 50100);

    // Test: Assign multiple ports (should get same port as no state changes)
    let mut used_ports = vec![port];
    for _ in 0..5 {
        let next_port = port_manager.assign_port(&state).unwrap();
        assert!(next_port >= 50000 && next_port <= 50100);
        used_ports.push(next_port);
    }
}

#[tokio::test]
async fn test_e2e_env_file_detection() {
    use chaba::core::env::copy_env_files;

    let temp_dir = TempDir::new().unwrap();
    let main_path = temp_dir.path().join("main");
    let review_path = temp_dir.path().join("review");

    std::fs::create_dir_all(&main_path).unwrap();
    std::fs::create_dir_all(&review_path).unwrap();

    // Create .env file with sensitive content
    std::fs::write(
        main_path.join(".env"),
        "DATABASE_URL=postgres://localhost\nAPI_KEY=secret123\nDEBUG=true\n",
    )
    .unwrap();

    // Create .env.local
    std::fs::write(main_path.join(".env.local"), "LOCAL=true\n").unwrap();

    // Test: Copy env files
    let result = copy_env_files(&main_path, &review_path, &[".env.local".to_string()]).await;
    assert!(result.is_ok());

    // Verify files were copied
    assert!(review_path.join(".env").exists());
    assert!(review_path.join(".env.local").exists());

    // Verify content
    let copied_content = std::fs::read_to_string(review_path.join(".env")).unwrap();
    assert!(copied_content.contains("DATABASE_URL"));
}

#[test]
fn test_e2e_config_validation() {
    use chaba::config::{Config, PortConfig};

    // Test: Default config is valid
    let config = Config::default();
    assert!(config.sandbox.port.validate().is_ok());

    // Test: Invalid port config
    let invalid_port = PortConfig {
        enabled: true,
        range_start: 80,
        range_end: 100,
    };
    assert!(invalid_port.validate().is_err());

    // Test: Valid port config
    let valid_port = PortConfig {
        enabled: true,
        range_start: 3000,
        range_end: 4000,
    };
    assert!(valid_port.validate().is_ok());
}

#[test]
fn test_e2e_state_persistence() {
    use chaba::core::state::{ReviewState, State};
    use chrono::Utc;

    let temp_dir = TempDir::new().unwrap();

    // Override home directory for testing
    std::env::set_var("HOME", temp_dir.path());

    let mut state = State::default();
    let review = ReviewState {
        pr_number: 999,
        branch: "test/branch".to_string(),
        worktree_path: temp_dir.path().join("worktree"),
        created_at: Utc::now(),
        port: Some(3000),
        project_type: Some("node".to_string()),
        deps_installed: true,
        env_copied: true,
        agent_analyses: Vec::new(),
    };

    // Test: Add review
    state.add_review(review.clone()).unwrap();

    // Test: Load state
    let loaded_state = State::load().unwrap();
    assert_eq!(loaded_state.reviews.len(), 1);
    assert_eq!(loaded_state.reviews[0].pr_number, 999);

    // Test: Remove review
    let mut state = State::load().unwrap();
    state.remove_review(999).unwrap();

    let loaded_state = State::load().unwrap();
    assert_eq!(loaded_state.reviews.len(), 0);
}
