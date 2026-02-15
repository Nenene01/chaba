use dialoguer::Confirm;

use crate::config::Config;
use crate::core::state::State;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;

pub async fn execute(pr: u32, force: bool) -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;

    println!("🍵 Chaba - Cleaning up review environment...\n");

    // Get review info for confirmation
    let state = State::load()?;
    if let Some(review) = state.get_review(pr) {
        println!("Review environment for PR #{}:", pr);
        println!("  Branch: {}", review.branch);
        println!("  Path: {}", review.worktree_path.display());

        // Interactive confirmation (unless --force/--yes is specified)
        if !force {
            let confirmed = Confirm::new()
                .with_prompt("Are you sure you want to remove this worktree?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !confirmed {
                println!("Cleanup cancelled.");
                return Ok(());
            }
        }
    }

    manager.remove(pr).await?;

    println!("✓ Removed worktree for PR #{}", pr);
    println!("✨ Cleanup complete!");

    Ok(())
}
