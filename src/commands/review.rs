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

    if let Some(project_type) = &review.project_type {
        println!("✓ Detected project type: {}", project_type);
    }

    if review.deps_installed {
        println!("✓ Dependencies installed");
    }

    if review.env_copied {
        println!("✓ Environment files copied");
    }

    if let Some(port) = review.port {
        println!("✓ Assigned port: {}", port);
    }

    println!("\n✨ Ready to review!");
    println!("\nTo start reviewing:");
    println!("  cd {}", review.worktree_path.display());

    if let Some(port) = review.port {
        println!("  # Start dev server on port {}", port);
    }

    println!("  code .  # or your preferred editor");

    Ok(())
}
