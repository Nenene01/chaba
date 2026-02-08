use chaba::config::Config;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_config_default() {
    let config = Config::default();

    // Worktree config
    assert_eq!(config.worktree.naming_template, "pr-{pr}");
    assert_eq!(config.worktree.auto_cleanup, true);
    assert_eq!(config.worktree.keep_days, 7);

    // Sandbox config
    assert_eq!(config.sandbox.auto_install_deps, true);
    assert_eq!(config.sandbox.copy_env_from_main, true);
    assert_eq!(config.sandbox.node.package_manager, "auto");
    assert_eq!(config.sandbox.port.enabled, true);
    assert_eq!(config.sandbox.port.range_start, 3000);
    assert_eq!(config.sandbox.port.range_end, 4000);

    // Agents config
    assert_eq!(config.agents.enabled, true);
    assert_eq!(config.agents.default_agents, vec!["claude"]);
    assert_eq!(config.agents.thorough_agents, vec!["claude", "codex", "gemini"]);
    assert_eq!(config.agents.timeout, 600);
    assert_eq!(config.agents.parallel, true);
}

#[test]
fn test_config_example_generation() {
    let example = Config::example();

    assert!(!example.is_empty());
    assert!(example.contains("worktree:"));
    assert!(example.contains("sandbox:"));
    assert!(example.contains("agents:"));
}

#[test]
fn test_config_yaml_serialization() {
    let config = Config::default();
    let yaml = serde_yaml::to_string(&config).unwrap();

    assert!(yaml.contains("naming_template: pr-{pr}"));
    assert!(yaml.contains("auto_cleanup: true"));
    assert!(yaml.contains("package_manager: auto"));
    assert!(yaml.contains("enabled: true"));
    assert!(yaml.contains("timeout: 600"));
}

#[test]
fn test_config_yaml_deserialization() {
    let yaml = r#"
worktree:
  base_dir: /custom/path
  naming_template: "review-{pr}"
  auto_cleanup: false
  keep_days: 14

sandbox:
  auto_install_deps: false
  copy_env_from_main: false
  additional_env_files:
    - .env.production
  node:
    package_manager: pnpm
  port:
    enabled: false
    range_start: 4000
    range_end: 5000

agents:
  enabled: false
  default_agents:
    - codex
  thorough_agents:
    - claude
    - codex
  timeout: 300
  parallel: false
"#;

    let config: Config = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(config.worktree.naming_template, "review-{pr}");
    assert_eq!(config.worktree.auto_cleanup, false);
    assert_eq!(config.worktree.keep_days, 14);

    assert_eq!(config.sandbox.auto_install_deps, false);
    assert_eq!(config.sandbox.copy_env_from_main, false);
    assert_eq!(config.sandbox.additional_env_files, vec![".env.production"]);
    assert_eq!(config.sandbox.node.package_manager, "pnpm");
    assert_eq!(config.sandbox.port.enabled, false);
    assert_eq!(config.sandbox.port.range_start, 4000);
    assert_eq!(config.sandbox.port.range_end, 5000);

    assert_eq!(config.agents.enabled, false);
    assert_eq!(config.agents.default_agents, vec!["codex"]);
    assert_eq!(config.agents.thorough_agents, vec!["claude", "codex"]);
    assert_eq!(config.agents.timeout, 300);
    assert_eq!(config.agents.parallel, false);
}

#[test]
fn test_config_partial_yaml() {
    // Only specify worktree, others should use defaults
    let yaml = r#"
worktree:
  naming_template: "custom-{pr}"
"#;

    let config: Config = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(config.worktree.naming_template, "custom-{pr}");
    assert_eq!(config.worktree.auto_cleanup, true); // default

    // Sandbox should use defaults
    assert_eq!(config.sandbox.auto_install_deps, true);

    // Agents should use defaults
    assert_eq!(config.agents.enabled, true);
}

#[test]
fn test_config_load_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Should return default config when no config file exists
    let config = Config::load().unwrap();

    assert_eq!(config.worktree.naming_template, "pr-{pr}");
    assert_eq!(config.agents.enabled, true);
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chaba.yaml");

    let mut config = Config::default();
    config.worktree.naming_template = "test-{pr}".to_string();
    config.agents.timeout = 1200;

    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();

    // Change to temp dir to load the config
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let loaded = Config::load().unwrap();
    assert_eq!(loaded.worktree.naming_template, "test-{pr}");
    assert_eq!(loaded.agents.timeout, 1200);
}
