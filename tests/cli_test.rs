use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Chaba"))
        .stdout(predicate::str::contains("茶葉"))
        .stdout(predicate::str::contains("review"))
        .stdout(predicate::str::contains("cleanup"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("agent-result"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("chaba"));
}

#[test]
fn test_review_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("review").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start a review environment"))
        .stdout(predicate::str::contains("--pr"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--worktree"))
        .stdout(predicate::str::contains("--with-agent"))
        .stdout(predicate::str::contains("--thorough"));
}

#[test]
fn test_cleanup_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("cleanup").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Clean up a review environment"))
        .stdout(predicate::str::contains("--pr"));
}

#[test]
fn test_list_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List active review environments"));
}

#[test]
fn test_status_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("status").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show status of a review environment"))
        .stdout(predicate::str::contains("--pr"));
}

#[test]
fn test_config_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("config").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize configuration"))
        .stdout(predicate::str::contains("--local"));
}

#[test]
fn test_agent_result_command_help() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("agent-result").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("View AI agent analysis results"))
        .stdout(predicate::str::contains("--pr"));
}

#[test]
fn test_review_command_missing_args() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("review");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_cleanup_command_missing_pr() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("cleanup");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_status_command_missing_pr() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("status");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_agent_result_command_missing_pr() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("agent-result");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_review_pr_and_branch_conflict() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("review")
        .arg("--pr")
        .arg("123")
        .arg("--branch")
        .arg("feature/test");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("conflict").or(predicate::str::contains("cannot be used")));
}

#[test]
fn test_list_command_basic() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("list");

    // Should succeed even with no active reviews
    cmd.assert().success();
}

#[test]
fn test_config_command_local() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("chaba").unwrap();
    cmd.current_dir(temp_dir.path()).arg("config").arg("--local");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    // Verify config file was created
    assert!(temp_dir.path().join("chaba.yaml").exists());
}

#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("--verbose").arg("list");

    cmd.assert().success();
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("chaba").unwrap();

    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}
