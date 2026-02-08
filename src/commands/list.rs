use crate::config::Config;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;
use chrono::Local;

pub async fn execute() -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;

    let reviews = manager.list()?;

    if reviews.is_empty() {
        println!("No active review environments.");
        return Ok(());
    }

    println!("Active review environments:\n");
    println!("{:<8} {:<50} {:<30} {}", "PR #", "Path", "Branch", "Created");
    println!("{}", "-".repeat(120));

    for review in reviews {
        let time_ago = format_time_ago(review.created_at);

        println!(
            "{:<8} {:<50} {:<30} {}",
            review.pr_number,
            review.worktree_path.display(),
            review.branch,
            time_ago
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
