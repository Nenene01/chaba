use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub worktree: WorktreeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Base directory for creating worktrees
    #[serde(default = "default_base_dir")]
    pub base_dir: PathBuf,

    /// Naming template for worktrees (e.g., "pr-{pr}")
    #[serde(default = "default_naming_template")]
    pub naming_template: String,

    /// Auto cleanup old worktrees
    #[serde(default = "default_auto_cleanup")]
    pub auto_cleanup: bool,

    /// Days to keep worktrees before auto cleanup
    #[serde(default = "default_keep_days")]
    pub keep_days: u32,
}

fn default_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("reviews")
}

fn default_naming_template() -> String {
    "pr-{pr}".to_string()
}

fn default_auto_cleanup() -> bool {
    true
}

fn default_keep_days() -> u32 {
    7
}

impl Default for Config {
    fn default() -> Self {
        Config {
            worktree: WorktreeConfig::default(),
        }
    }
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        WorktreeConfig {
            base_dir: default_base_dir(),
            naming_template: default_naming_template(),
            auto_cleanup: default_auto_cleanup(),
            keep_days: default_keep_days(),
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // Try to load from current directory first
        if let Ok(config) = Self::load_from_path("chaba.yaml") {
            return Ok(config);
        }

        // Try user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("chaba").join("chaba.yaml");
            if let Ok(config) = Self::load_from_path(&config_path) {
                return Ok(config);
            }
        }

        // Use default configuration
        Ok(Config::default())
    }

    fn load_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Generate example configuration
    pub fn example() -> String {
        let config = Config::default();
        serde_yaml::to_string(&config).unwrap_or_else(|_| String::from("# Failed to generate config"))
    }
}
