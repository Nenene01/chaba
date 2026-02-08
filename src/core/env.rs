use std::path::Path;
use tokio::fs;

use crate::error::{ChabaError, Result};

/// Patterns that may indicate sensitive information
const SENSITIVE_PATTERNS: &[&str] = &[
    "PASSWORD",
    "SECRET",
    "PRIVATE_KEY",
    "API_KEY",
    "TOKEN",
    "CREDENTIAL",
    "AUTH",
];

/// Check if a file contains potentially sensitive information
async fn check_sensitive_content(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).await?;
    let mut warnings = Vec::new();

    for line in content.lines() {
        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for sensitive patterns
        for pattern in SENSITIVE_PATTERNS {
            if line.to_uppercase().contains(pattern) {
                // Extract variable name
                if let Some(var_name) = line.split('=').next() {
                    warnings.push(var_name.trim().to_string());
                    break;
                }
            }
        }
    }

    Ok(warnings)
}

/// Validate that a file path is safe (no symlinks outside the base directory)
fn validate_file_path(file_path: &Path, base_dir: &Path) -> Result<()> {
    // Resolve to canonical path (follows symlinks)
    let canonical_file = file_path.canonicalize()
        .map_err(|e| ChabaError::ConfigError(
            format!("Failed to resolve file path {}: {}", file_path.display(), e)
        ))?;

    let canonical_base = base_dir.canonicalize()
        .map_err(|e| ChabaError::ConfigError(
            format!("Failed to resolve base directory {}: {}", base_dir.display(), e)
        ))?;

    // Ensure the canonical file path is within the base directory
    if !canonical_file.starts_with(&canonical_base) {
        return Err(ChabaError::ConfigError(
            format!("Security error: {} is outside base directory {} (possible symlink attack)",
                canonical_file.display(), canonical_base.display())
        ));
    }

    Ok(())
}

/// Copy environment files from main worktree to review worktree
///
/// This function will:
/// 1. Check for potentially sensitive information
/// 2. Warn the user about sensitive variables
/// 3. Copy the files to the review environment
pub async fn copy_env_files(
    main_worktree: &Path,
    review_worktree: &Path,
    additional_files: &[String],
) -> Result<()> {
    let mut files = vec![".env".to_string()];
    files.extend_from_slice(additional_files);

    let mut copied_count = 0;
    let mut has_warnings = false;

    for file in files {
        let src = main_worktree.join(&file);
        if src.exists() {
            // Validate source file is within main_worktree (prevent symlink attacks)
            validate_file_path(&src, main_worktree)?;

            // Check for sensitive content
            if let Ok(warnings) = check_sensitive_content(&src).await {
                if !warnings.is_empty() {
                    if !has_warnings {
                        eprintln!("⚠️  Warning: Potentially sensitive information detected");
                        eprintln!("The following variables may contain secrets:");
                        has_warnings = true;
                    }
                    eprintln!("\n  In {}:", file);
                    for var in &warnings {
                        eprintln!("    - {}", var);
                    }
                }
            }

            let dst = review_worktree.join(&file);

            // Ensure destination directory exists
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).await?;
            }

            // Copy file (not following symlinks)
            fs::copy(&src, &dst).await?;
            tracing::info!("Copied {} to review environment", file);
            copied_count += 1;
        }
    }

    if has_warnings {
        eprintln!("\n💡 Tip: Consider using .env.example for review environments");
        eprintln!("   or set copy_env_from_main=false in your config");
    }

    if copied_count > 0 {
        tracing::info!("Copied {} environment file(s)", copied_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_copy_env_files() {
        let main_dir = TempDir::new().unwrap();
        let review_dir = TempDir::new().unwrap();

        // Create .env file
        write(main_dir.path().join(".env"), "API_KEY=secret").await.unwrap();

        // Copy files
        copy_env_files(main_dir.path(), review_dir.path(), &[]).await.unwrap();

        // Verify
        assert!(review_dir.path().join(".env").exists());
        let content = fs::read_to_string(review_dir.path().join(".env")).await.unwrap();
        assert_eq!(content, "API_KEY=secret");
    }

    #[tokio::test]
    async fn test_copy_additional_files() {
        let main_dir = TempDir::new().unwrap();
        let review_dir = TempDir::new().unwrap();

        // Create files
        write(main_dir.path().join(".env"), "API_KEY=secret").await.unwrap();
        write(main_dir.path().join(".env.local"), "DEBUG=true").await.unwrap();

        // Copy files
        copy_env_files(
            main_dir.path(),
            review_dir.path(),
            &[".env.local".to_string()],
        )
        .await
        .unwrap();

        // Verify both files copied
        assert!(review_dir.path().join(".env").exists());
        assert!(review_dir.path().join(".env.local").exists());
    }
}
