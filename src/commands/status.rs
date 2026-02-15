use crate::core::git::GitOps;
use crate::core::state::State;
use crate::error::{ChabaError, Result};
use chrono::Local;

pub async fn execute(pr: u32) -> Result<()> {
    let state = State::load()?;
    let review = state
        .get_review(pr)
        .ok_or(ChabaError::WorktreeNotFound(pr))?;

    let git_ops = GitOps::open()?;

    println!("🍵 Review Environment Status\n");
    println!("PR Number:     #{}", review.pr_number);
    println!("Branch:        {}", review.branch);
    println!("Path:          {}", review.worktree_path.display());

    // Check if worktree actually exists
    let worktree_exists = review.worktree_path.exists();
    if !worktree_exists {
        println!("Status:        ⚠️  MISSING (worktree was manually removed)");
        println!("\n💡 Tip: Run 'chaba cleanup --force --pr {}' to clean up the state.", pr);
    } else {
        println!("Status:        ✓ Active");
    }

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

    // Show Git statistics if worktree exists
    if worktree_exists {
        if let Ok(stats) = git_ops.get_stats(&review.worktree_path).await {
            println!("\nGit Status:");

            if let Some(ref upstream) = stats.upstream_branch {
                println!("  Upstream:     {}", upstream);
            }

            if stats.files_changed > 0 || stats.lines_added > 0 || stats.lines_deleted > 0 {
                println!(
                    "  Changes:      {} file(s), +{} -{} lines",
                    stats.files_changed, stats.lines_added, stats.lines_deleted
                );
            } else {
                println!("  Changes:      No uncommitted changes");
            }

            if stats.commits_ahead > 0 || stats.commits_behind > 0 {
                let mut status_parts = Vec::new();
                if stats.commits_ahead > 0 {
                    status_parts.push(format!("↑{} ahead", stats.commits_ahead));
                }
                if stats.commits_behind > 0 {
                    status_parts.push(format!("↓{} behind", stats.commits_behind));
                }
                println!("  Commits:      {}", status_parts.join(", "));
            } else if stats.upstream_branch.is_some() {
                println!("  Commits:      Up to date");
            }
        }
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
