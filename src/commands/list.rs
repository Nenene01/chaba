use crate::config::Config;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;

pub async fn execute() -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;

    let reviews = manager.list()?;

    if reviews.is_empty() {
        println!("No active review environments.");
        return Ok(());
    }

    // Check for missing worktrees and collect stale entries
    let mut stale_prs = Vec::new();
    for review in &reviews {
        if !review.worktree_path.exists() {
            stale_prs.push(review.pr_number);
        }
    }

    // Warn about stale entries
    if !stale_prs.is_empty() {
        eprintln!("⚠️  Warning: Found {} stale worktree(s) that no longer exist:", stale_prs.len());
        for pr in &stale_prs {
            eprintln!("    PR #{} - worktree was manually removed", pr);
        }
        eprintln!("\n💡 Tip: Run 'chaba cleanup --force --pr <PR>' to clean up the state.\n");
    }

    println!("Active review environments:\n");
    println!("{:<8} {:<50} {:<30} {:<15} {}", "PR #", "Path", "Branch", "Created", "Status");
    println!("{}", "-".repeat(130));

    for review in reviews {
        let time_ago = format_time_ago(review.created_at);
        let status = if review.worktree_path.exists() {
            "✓"
        } else {
            "⚠️  MISSING"
        };

        println!(
            "{:<8} {:<50} {:<30} {:<15} {}",
            review.pr_number,
            review.worktree_path.display(),
            review.branch,
            time_ago,
            status
        );
    }

    Ok(())
}

fn format_time_ago(created_at: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(created_at);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}
