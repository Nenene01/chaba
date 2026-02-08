use std::path::Path;
use tokio::process::Command;

use crate::core::project::{NodePackageManager, ProjectType};
use crate::error::Result;

/// Install dependencies for the given project type
pub async fn install_dependencies(
    worktree_path: &Path,
    project_type: &ProjectType,
) -> Result<()> {
    match project_type {
        ProjectType::NodeJs { package_manager } => {
            install_node_deps(worktree_path, package_manager).await
        }
        ProjectType::Rust => install_rust_deps(worktree_path).await,
        ProjectType::Python {
            has_requirements,
            has_pyproject,
        } => install_python_deps(worktree_path, *has_requirements, *has_pyproject).await,
        ProjectType::Go => install_go_deps(worktree_path).await,
        ProjectType::Unknown => {
            tracing::info!("Unknown project type, skipping dependency installation");
            Ok(())
        }
    }
}

/// Install Node.js dependencies
async fn install_node_deps(path: &Path, pm: &NodePackageManager) -> Result<()> {
    tracing::info!("Installing Node.js dependencies using {}...", pm.as_str());

    let parts: Vec<&str> = pm.install_command().split_whitespace().collect();
    let (cmd, args) = parts.split_first().unwrap();

    let output = Command::new(cmd)
        .args(args)
        .current_dir(path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Failed to install dependencies: {}", error);
        return Err(crate::error::ChabaError::Other(anyhow::anyhow!(
            "Dependency installation failed: {}",
            error
        )));
    }

    tracing::info!("Dependencies installed successfully");
    Ok(())
}

/// Install Rust dependencies
async fn install_rust_deps(path: &Path) -> Result<()> {
    tracing::info!("Building Rust project...");

    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Failed to build project: {}", error);
        return Err(crate::error::ChabaError::Other(anyhow::anyhow!(
            "Cargo build failed: {}",
            error
        )));
    }

    tracing::info!("Rust project built successfully");
    Ok(())
}

/// Install Python dependencies
async fn install_python_deps(
    path: &Path,
    has_requirements: bool,
    has_pyproject: bool,
) -> Result<()> {
    tracing::info!("Installing Python dependencies...");

    if has_requirements {
        let output = Command::new("pip")
            .args(["install", "-r", "requirements.txt"])
            .current_dir(path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Failed to install requirements: {}", error);
            return Err(crate::error::ChabaError::Other(anyhow::anyhow!(
                "pip install failed: {}",
                error
            )));
        }
    }

    if has_pyproject {
        let output = Command::new("pip")
            .args(["install", "-e", "."])
            .current_dir(path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to install pyproject: {}", error);
            // Don't fail if pyproject install fails
        }
    }

    tracing::info!("Python dependencies installed successfully");
    Ok(())
}

/// Install Go dependencies
async fn install_go_deps(path: &Path) -> Result<()> {
    tracing::info!("Downloading Go modules...");

    let output = Command::new("go")
        .args(["mod", "download"])
        .current_dir(path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Failed to download modules: {}", error);
        return Err(crate::error::ChabaError::Other(anyhow::anyhow!(
            "go mod download failed: {}",
            error
        )));
    }

    tracing::info!("Go modules downloaded successfully");
    Ok(())
}
