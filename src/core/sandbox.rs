use std::path::Path;

use crate::config::SandboxConfig;
use crate::core::{env, installer, port::PortManager, project, state::State};
use crate::error::Result;

pub struct SandboxManager {
    config: SandboxConfig,
}

#[derive(Debug, Default)]
pub struct SandboxInfo {
    pub project_type: Option<String>,
    pub deps_installed: bool,
    pub env_copied: bool,
    pub port: Option<u16>,
}

impl SandboxManager {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Set up sandbox environment for a review worktree
    pub async fn setup(
        &self,
        worktree_path: &Path,
        main_worktree: &Path,
        state: &State,
    ) -> Result<SandboxInfo> {
        let mut info = SandboxInfo::default();

        tracing::info!("Setting up sandbox environment...");

        // 1. Detect project type
        let project_type = project::detect_project_type(worktree_path)?;
        info.project_type = Some(project_type.as_string());
        tracing::info!("Detected project type: {}", project_type.as_string());

        // 2. Install dependencies
        if self.config.auto_install_deps {
            tracing::info!("Installing dependencies...");
            match installer::install_dependencies(worktree_path, &project_type).await {
                Ok(_) => {
                    info.deps_installed = true;
                    tracing::info!("Dependencies installed successfully");
                }
                Err(e) => {
                    tracing::warn!("Failed to install dependencies: {}", e);
                    // Continue even if installation fails
                }
            }
        }

        // 3. Copy environment files
        if self.config.copy_env_from_main {
            tracing::info!("Copying environment files...");
            match env::copy_env_files(
                main_worktree,
                worktree_path,
                &self.config.additional_env_files,
            )
            .await
            {
                Ok(_) => {
                    info.env_copied = true;
                    tracing::info!("Environment files copied");
                }
                Err(e) => {
                    tracing::warn!("Failed to copy environment files: {}", e);
                    // Continue even if copy fails
                }
            }
        }

        // 4. Assign port
        if self.config.port.enabled {
            let port_manager = PortManager::new(
                self.config.port.range_start,
                self.config.port.range_end,
            );

            match port_manager.assign_port(state) {
                Ok(port) => {
                    info.port = Some(port);
                    tracing::info!("Assigned port: {}", port);
                }
                Err(e) => {
                    tracing::warn!("Failed to assign port: {}", e);
                    // Continue even if port assignment fails
                }
            }
        }

        tracing::info!("Sandbox environment setup complete");
        Ok(info)
    }
}
