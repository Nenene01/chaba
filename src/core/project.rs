use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    NodeJs { package_manager: NodePackageManager },
    Rust,
    Python { has_requirements: bool, has_pyproject: bool },
    Go,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl ProjectType {
    pub fn as_string(&self) -> String {
        match self {
            ProjectType::NodeJs { package_manager } => {
                format!("Node.js ({})", package_manager.as_str())
            }
            ProjectType::Rust => "Rust".to_string(),
            ProjectType::Python { .. } => "Python".to_string(),
            ProjectType::Go => "Go".to_string(),
            ProjectType::Unknown => "Unknown".to_string(),
        }
    }
}

impl NodePackageManager {
    pub fn as_str(&self) -> &str {
        match self {
            NodePackageManager::Npm => "npm",
            NodePackageManager::Yarn => "yarn",
            NodePackageManager::Pnpm => "pnpm",
            NodePackageManager::Bun => "bun",
        }
    }

    pub fn install_command(&self) -> &str {
        match self {
            NodePackageManager::Npm => "npm install",
            NodePackageManager::Yarn => "yarn install",
            NodePackageManager::Pnpm => "pnpm install",
            NodePackageManager::Bun => "bun install",
        }
    }
}

/// Detect project type from worktree path
pub fn detect_project_type(path: &Path) -> Result<ProjectType> {
    // Check for Node.js
    if path.join("package.json").exists() {
        let pm = detect_node_package_manager(path);
        return Ok(ProjectType::NodeJs { package_manager: pm });
    }

    // Check for Rust
    if path.join("Cargo.toml").exists() {
        return Ok(ProjectType::Rust);
    }

    // Check for Python
    let has_requirements = path.join("requirements.txt").exists();
    let has_pyproject = path.join("pyproject.toml").exists();
    if has_requirements || has_pyproject {
        return Ok(ProjectType::Python {
            has_requirements,
            has_pyproject,
        });
    }

    // Check for Go
    if path.join("go.mod").exists() {
        return Ok(ProjectType::Go);
    }

    Ok(ProjectType::Unknown)
}

/// Detect Node.js package manager
fn detect_node_package_manager(path: &Path) -> NodePackageManager {
    // Check for lock files in priority order
    if path.join("bun.lockb").exists() {
        return NodePackageManager::Bun;
    }

    if path.join("pnpm-lock.yaml").exists() {
        return NodePackageManager::Pnpm;
    }

    if path.join("yarn.lock").exists() {
        return NodePackageManager::Yarn;
    }

    // Default to npm
    NodePackageManager::Npm
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_nodejs_npm() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("package-lock.json"), "{}").unwrap();

        let project_type = detect_project_type(dir.path()).unwrap();
        assert!(matches!(
            project_type,
            ProjectType::NodeJs {
                package_manager: NodePackageManager::Npm
            }
        ));
    }

    #[test]
    fn test_detect_nodejs_yarn() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("yarn.lock"), "").unwrap();

        let project_type = detect_project_type(dir.path()).unwrap();
        assert!(matches!(
            project_type,
            ProjectType::NodeJs {
                package_manager: NodePackageManager::Yarn
            }
        ));
    }

    #[test]
    fn test_detect_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

        let project_type = detect_project_type(dir.path()).unwrap();
        assert!(matches!(project_type, ProjectType::Rust));
    }

    #[test]
    fn test_detect_unknown() {
        let dir = TempDir::new().unwrap();

        let project_type = detect_project_type(dir.path()).unwrap();
        assert!(matches!(project_type, ProjectType::Unknown));
    }
}
