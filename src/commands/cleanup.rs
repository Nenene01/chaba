use crate::config::Config;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;

pub async fn execute(pr: u32) -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;

    println!("🍵 Chaba - Cleaning up review environment...\n");

    manager.remove(pr).await?;

    println!("✓ Removed worktree for PR #{}", pr);
    println!("✨ Cleanup complete!");

    Ok(())
}
