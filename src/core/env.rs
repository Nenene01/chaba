use std::path::Path;
use tokio::fs;

use crate::error::Result;

/// Copy environment files from main worktree to review worktree
pub async fn copy_env_files(
    main_worktree: &Path,
    review_worktree: &Path,
    additional_files: &[String],
) -> Result<()> {
    let mut files = vec![".env".to_string()];
    files.extend_from_slice(additional_files);

    let mut copied_count = 0;

    for file in files {
        let src = main_worktree.join(&file);
        if src.exists() {
            let dst = review_worktree.join(&file);
            fs::copy(&src, &dst).await?;
            tracing::info!("Copied {} to review environment", file);
            copied_count += 1;
        }
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
