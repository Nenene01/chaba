use crate::core::git::GitOps;
use crate::core::state::State;
use crate::error::{ChabaError, Result};

pub async fn execute(pr: u32, from_branch: String) -> Result<()> {
    let state = State::load()?;
    let review = state
        .get_review(pr)
        .ok_or(ChabaError::WorktreeNotFound(pr))?;

    println!("🍵 Chaba - Merging branch into worktree...\n");
    println!("PR #:         {}", pr);
    println!("Worktree:     {}", review.worktree_path.display());
    println!("Current:      {}", review.branch);
    println!("Merging from: {}\n", from_branch);

    // Verify worktree exists
    if !review.worktree_path.exists() {
        return Err(ChabaError::Other(anyhow::anyhow!(
            "Worktree does not exist: {}",
            review.worktree_path.display()
        )));
    }

    let git_ops = GitOps::open()?;

    // Perform the merge
    println!("Merging...");
    git_ops.merge(&review.worktree_path, &from_branch).await?;

    println!("\n✓ Merge completed successfully!");
    println!("\nNext steps:");
    println!("  cd {}", review.worktree_path.display());
    println!("  git push  # Push the merged changes");

    Ok(())
}
