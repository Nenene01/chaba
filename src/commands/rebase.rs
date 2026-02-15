use crate::core::git::GitOps;
use crate::core::state::State;
use crate::error::{ChabaError, Result};

pub async fn execute(pr: u32, onto_branch: String) -> Result<()> {
    let state = State::load()?;
    let review = state
        .get_review(pr)
        .ok_or(ChabaError::WorktreeNotFound(pr))?;

    println!("🍵 Chaba - Rebasing worktree onto branch...\n");
    println!("PR #:        {}", pr);
    println!("Worktree:    {}", review.worktree_path.display());
    println!("Current:     {}", review.branch);
    println!("Rebasing onto: {}\n", onto_branch);

    // Verify worktree exists
    if !review.worktree_path.exists() {
        return Err(ChabaError::Other(anyhow::anyhow!(
            "Worktree does not exist: {}",
            review.worktree_path.display()
        )));
    }

    let git_ops = GitOps::open()?;

    // Perform the rebase
    println!("Rebasing...");
    git_ops.rebase(&review.worktree_path, &onto_branch).await?;

    println!("\n✓ Rebase completed successfully!");
    println!("\nNext steps:");
    println!("  cd {}", review.worktree_path.display());
    println!("  git push --force-with-lease  # Force push the rebased changes");

    Ok(())
}
