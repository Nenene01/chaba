use crate::config::Config;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;

pub async fn execute(
    pr: Option<u32>,
    branch: Option<String>,
    force: bool,
    worktree: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;

    println!("🍵 Chaba - Creating review environment...\n");

    let review = manager.create(pr, branch.clone(), force, worktree).await?;

    println!("✓ Fetched branch: {}", review.branch);
    println!("✓ Created worktree at: {}", review.worktree_path.display());
    println!("\n✨ Ready to review!");
    println!("\nTo start reviewing:");
    println!("  cd {}", review.worktree_path.display());
    println!("  code .  # or your preferred editor");

    Ok(())
}
