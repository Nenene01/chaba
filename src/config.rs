use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub worktree: WorktreeConfig,

    #[serde(default)]
    pub sandbox: SandboxConfig,

    #[serde(default)]
    pub agents: AgentsConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    /// Enable AI agent integration
    #[serde(default = "default_agents_enabled")]
    pub enabled: bool,

    /// Default agents for --with-agent
    #[serde(default = "default_default_agents")]
    pub default_agents: Vec<String>,

    /// Agents for --thorough review
    #[serde(default = "default_thorough_agents")]
    pub thorough_agents: Vec<String>,

    /// Timeout in seconds
    #[serde(default = "default_agent_timeout")]
    pub timeout: u64,

    /// Enable parallel execution
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
        Ok(config)
    }

    /// Generate example configuration
    pub fn example() -> String {
        let config = Config::default();
        serde_yaml::to_string(&config).unwrap_or_else(|_| String::from("# Failed to generate config"))
    }
}
