use crate::config::Config;
use crate::core::agent::AgentManager;
use crate::core::session::SessionManager;
use crate::core::state::State;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;
use std::path::PathBuf;

pub async fn execute(
    pr: Option<u32>,
    branch: Option<String>,
    force: bool,
    worktree: Option<String>,
    with_agent: bool,
    thorough: bool,
    copy_session_from: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let manager = WorktreeManager::new(config.clone())?;

    println!("🍵 Chaba - Creating review environment...\n");

    let mut review = manager.create(pr, branch.clone(), force, worktree).await?;

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

    // Copy session data if requested
    if let Some(source_path_str) = copy_session_from {
        println!("\n📋 Copying Claude Code session data...");

        let session_manager = SessionManager::new()?;
        let source_path = PathBuf::from(source_path_str);
        let target_path = &review.worktree_path;

        match session_manager.copy_session_data(&source_path, target_path).await {
            Ok(true) => {
                println!("✓ Session data copied successfully");
            }
            Ok(false) => {
                println!("⚠️  No session data found at source path");
            }
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to copy session data: {}", e);
                eprintln!("   Continuing with worktree creation...");
            }
        }
    }

    // Run AI agents if requested
    let run_agents = if with_agent || thorough {
        true
    } else if config.agents.enabled {
        // Interactive mode: ask if user wants to run agents
        use dialoguer::Confirm;

        Confirm::new()
            .with_prompt("Run AI agent analysis?")
            .default(false)
            .interact()
            .unwrap_or(false)
    } else {
        false
    };

    if run_agents {
        println!("\n🤖 Running AI agent analysis...");

        let agent_manager = AgentManager::new(config.agents);
        let pr_number = review.pr_number;
        let analyses = agent_manager
            .run_review(pr_number, &review.worktree_path, thorough)
            .await?;

        if !analyses.is_empty() {
            println!("✓ Completed analysis with {} agent(s)", analyses.len());

            // Save analyses to state
            review.agent_analyses = analyses;
            let mut state = State::load()?;
            state.add_review(review.clone())?;

            println!("\nRun 'chaba agent-result {}' to view detailed results", pr_number);
        }
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
