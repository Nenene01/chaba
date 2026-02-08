use crate::core::state::State;
use crate::error::{ChabaError, Result};
use chrono::Local;

pub async fn execute(pr: u32) -> Result<()> {
    let state = State::load()?;
    let review = state
        .get_review(pr)
        .ok_or(ChabaError::WorktreeNotFound(pr))?;

    println!("🍵 Review Environment Status\n");
    println!("PR Number:     #{}", review.pr_number);
    println!("Branch:        {}", review.branch);
    println!("Path:          {}", review.worktree_path.display());

    let created = review.created_at.with_timezone(&Local);
    let time_ago = format_time_ago(review.created_at);
    println!("Created:       {} ({})", created.format("%Y-%m-%d %H:%M:%S"), time_ago);

    if let Some(project_type) = &review.project_type {
        println!("\nProject Type:  {}", project_type);
    }

    if let Some(port) = review.port {
        println!("Port:          {} (http://localhost:{})", port, port);
    }

    println!("\nSandbox Setup:");
    println!("  Dependencies: {}", if review.deps_installed { "✓ Installed" } else { "✗ Not installed" });
    println!("  Environment:  {}", if review.env_copied { "✓ Copied" } else { "✗ Not copied" });

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
