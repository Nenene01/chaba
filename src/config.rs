//! Configuration management for Chaba.
//!
//! This module provides configuration structures for all Chaba features:
//! - Worktree management settings
//! - Sandbox environment configuration
//! - AI agent integration settings
//!
//! # Configuration File Locations
//!
//! Chaba looks for configuration in the following order:
//! 1. `./chaba.yaml` (current directory)
//! 2. `~/.config/chaba/chaba.yaml` (user config directory)
//! 3. Default values (if no config file exists)
//!
//! # Example Configuration
//!
//! ```yaml
//! worktree:
//!   base_dir: ~/reviews
//!   naming_template: pr-{pr}
//!   auto_cleanup: true
//!   keep_days: 7
//!
//! sandbox:
//!   auto_install_deps: true
//!   copy_env_from_main: true
//!   node:
//!     package_manager: auto
//!   port:
//!     enabled: true
//!     range_start: 3000
//!     range_end: 4000
//!
//! agents:
//!   enabled: true
//!   default_agents:
//!     - claude
//!   thorough_agents:
//!     - claude
//!     - codex
//!     - gemini
//!   timeout: 600
//!   parallel: true
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

/// Main configuration structure for Chaba.
///
/// Contains all configuration sections for worktree management,
/// sandbox environment setup, and AI agent integration.
///
/// # Examples
///
/// ```rust
/// use chaba::Config;
///
/// // Load configuration from file or use defaults
/// let config = Config::load().unwrap();
///
/// // Access worktree configuration
/// println!("Base directory: {}", config.worktree.base_dir.display());
///
/// // Access sandbox configuration
/// println!("Auto install deps: {}", config.sandbox.auto_install_deps);
///
/// // Access agents configuration
/// println!("Agents enabled: {}", config.agents.enabled);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Worktree management settings
    #[serde(default)]
    pub worktree: WorktreeConfig,

    /// Sandbox environment settings
    #[serde(default)]
    pub sandbox: SandboxConfig,

    /// AI agent integration settings
    #[serde(default)]
    pub agents: AgentsConfig,
}

/// Configuration for git worktree management.
///
/// Controls where and how worktrees are created for PR review environments.
///
/// # Default Values
///
/// - `base_dir`: `~/reviews`
/// - `naming_template`: `"pr-{pr}"`
/// - `auto_cleanup`: `true`
/// - `keep_days`: `7`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Base directory for creating worktrees
    ///
    /// Default: `~/reviews`
    #[serde(default = "default_base_dir")]
    pub base_dir: PathBuf,

    /// Naming template for worktrees
    ///
    /// Use `{pr}` as placeholder for PR number.
    ///
    /// Default: `"pr-{pr}"`
    ///
    /// Examples: `"pr-{pr}"`, `"review-{pr}"`, `"feature-{pr}"`
    #[serde(default = "default_naming_template")]
    pub naming_template: String,

    /// Auto cleanup old worktrees
    ///
    /// Default: `true`
    #[serde(default = "default_auto_cleanup")]
    pub auto_cleanup: bool,

    /// Days to keep worktrees before auto cleanup
    ///
    /// Default: `7`
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Auto install dependencies
    #[serde(default = "default_auto_install_deps")]
    pub auto_install_deps: bool,

    /// Copy environment files from main worktree
    #[serde(default = "default_copy_env_from_main")]
    pub copy_env_from_main: bool,

    /// Additional environment files to copy
    #[serde(default)]
    pub additional_env_files: Vec<String>,

    /// Node.js configuration
    #[serde(default)]
    pub node: NodeConfig,

    /// Port configuration
    #[serde(default)]
    pub port: PortConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Package manager: auto, npm, yarn, pnpm, bun
    #[serde(default = "default_package_manager")]
    pub package_manager: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Enable automatic port assignment
    #[serde(default = "default_port_enabled")]
    pub enabled: bool,

    /// Port range start
    #[serde(default = "default_port_range_start")]
    pub range_start: u16,

    /// Port range end
    #[serde(default = "default_port_range_end")]
    pub range_end: u16,
}

fn default_auto_install_deps() -> bool {
    true
}

fn default_copy_env_from_main() -> bool {
    true
}

fn default_package_manager() -> String {
    "auto".to_string()
}

fn default_port_enabled() -> bool {
    true
}

fn default_port_range_start() -> u16 {
    3000
}

fn default_port_range_end() -> u16 {
    4000
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            auto_install_deps: default_auto_install_deps(),
            copy_env_from_main: default_copy_env_from_main(),
            additional_env_files: vec![".env.local".to_string()],
            node: NodeConfig::default(),
            port: PortConfig::default(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            package_manager: default_package_manager(),
        }
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        PortConfig {
            enabled: default_port_enabled(),
            range_start: default_port_range_start(),
            range_end: default_port_range_end(),
        }
    }
}

impl PortConfig {
    /// Validate port range configuration
    pub fn validate(&self) -> Result<()> {
        // Check if range_start < range_end
        if self.range_start >= self.range_end {
            return Err(crate::error::ChabaError::ConfigError(
                format!("Invalid port range: range_start ({}) must be less than range_end ({})",
                    self.range_start, self.range_end)
            ));
        }

        // Check if range_start is not in well-known port range (0-1023)
        if self.range_start < 1024 {
            return Err(crate::error::ChabaError::ConfigError(
                format!("Invalid port range: range_start ({}) should be >= 1024 (avoid well-known ports)",
                    self.range_start)
            ));
        }

        // Note: range_end is u16, so it's automatically <= 65535 (no check needed)

        // Check if range has at least some ports available (minimum 10 for safety)
        let range_size = self.range_end - self.range_start;
        if range_size < 10 {
            return Err(crate::error::ChabaError::ConfigError(
                format!("Invalid port range: range is too small ({} ports). Minimum 10 ports recommended.",
                    range_size)
            ));
        }

        Ok(())
    }
}

/// Configuration for AI agent integration.
///
/// Controls which AI agents are used for code review and how they execute.
///
/// # Available Agents
///
/// - `claude`: Claude Code (fast, general-purpose)
/// - `codex`: OpenAI Codex (implementation expert)
/// - `gemini`: Google Gemini (strategic analyst)
///
/// # Default Values
///
/// - `enabled`: `true`
/// - `default_agents`: `["claude"]`
/// - `thorough_agents`: `["claude", "codex", "gemini"]`
/// - `timeout`: `600` (10 minutes)
/// - `parallel`: `true`
///
/// # Examples
///
/// ```yaml
/// agents:
///   enabled: true
///   default_agents:
///     - claude
///   thorough_agents:
///     - claude
///     - codex
///     - gemini
///   timeout: 600
///   parallel: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    /// Enable AI agent integration
    ///
    /// Default: `true`
    #[serde(default = "default_agents_enabled")]
    pub enabled: bool,

    /// Default agents for `--with-agent` flag
    ///
    /// Used for quick reviews.
    ///
    /// Default: `["claude"]`
    #[serde(default = "default_default_agents")]
    pub default_agents: Vec<String>,

    /// Agents for `--thorough` flag
    ///
    /// Used for comprehensive multi-agent analysis.
    ///
    /// Default: `["claude", "codex", "gemini"]`
    #[serde(default = "default_thorough_agents")]
    pub thorough_agents: Vec<String>,

    /// Timeout in seconds for agent execution
    ///
    /// Default: `600` (10 minutes)
    #[serde(default = "default_agent_timeout")]
    pub timeout: u64,

    /// Enable parallel execution of agents
    ///
    /// When `true`, all agents run simultaneously.
    /// When `false`, agents run sequentially.
    ///
    /// Default: `true`
    #[serde(default = "default_parallel")]
    pub parallel: bool,
}

fn default_agents_enabled() -> bool {
    true
}

fn default_default_agents() -> Vec<String> {
    vec!["claude".to_string()]
}

fn default_thorough_agents() -> Vec<String> {
    vec![
        "claude".to_string(),
        "codex".to_string(),
        "gemini".to_string(),
    ]
}

fn default_agent_timeout() -> u64 {
    600
}

fn default_parallel() -> bool {
    true
}

impl Default for AgentsConfig {
    fn default() -> Self {
        AgentsConfig {
            enabled: default_agents_enabled(),
            default_agents: default_default_agents(),
            thorough_agents: default_thorough_agents(),
            timeout: default_agent_timeout(),
            parallel: default_parallel(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            worktree: WorktreeConfig::default(),
            sandbox: SandboxConfig::default(),
            agents: AgentsConfig::default(),
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

        // Validate port configuration
        config.sandbox.port.validate()?;

        Ok(config)
    }

    /// Generate example configuration
    pub fn example() -> String {
        let config = Config::default();
        serde_yaml::to_string(&config).unwrap_or_else(|_| String::from("# Failed to generate config"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_config_valid() {
        let config = PortConfig {
            enabled: true,
            range_start: 3000,
            range_end: 4000,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_port_config_start_greater_than_end() {
        let config = PortConfig {
            enabled: true,
            range_start: 4000,
            range_end: 3000,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be less than"));
    }

    #[test]
    fn test_port_config_well_known_ports() {
        let config = PortConfig {
            enabled: true,
            range_start: 80,
            range_end: 4000,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("should be >= 1024"));
    }

    #[test]
    fn test_port_config_boundary_values() {
        // Test with maximum valid port
        let config = PortConfig {
            enabled: true,
            range_start: 60000,
            range_end: 65535,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_port_config_range_too_small() {
        let config = PortConfig {
            enabled: true,
            range_start: 3000,
            range_end: 3005, // Only 5 ports
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("range is too small"));
    }

    #[test]
    fn test_port_config_minimum_range() {
        let config = PortConfig {
            enabled: true,
            range_start: 3000,
            range_end: 3010, // Exactly 10 ports
        };
        assert!(config.validate().is_ok());
    }
}
