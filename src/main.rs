use chaba::commands;
use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(
    name = "chaba",
    version,
    about = "AI Agent Friendly Source Review & Debug Environment",
    long_about = "Chaba (茶葉) - Integrates git worktree, branch operations, and sandbox environments for seamless team collaboration."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a review environment for a PR or branch
    Review {
        /// Pull request number
        #[arg(short, long, conflicts_with = "branch")]
        pr: Option<u32>,

        /// Branch name
        #[arg(short, long, conflicts_with = "pr")]
        branch: Option<String>,

        /// Force creation even if worktree exists
        #[arg(short, long)]
        force: bool,

        /// Custom worktree path
        #[arg(long)]
        worktree: Option<String>,

        /// Run AI agent analysis (uses default agents from config)
        #[arg(long)]
        with_agent: bool,

        /// Run thorough AI agent analysis (uses all configured agents)
        #[arg(long)]
        thorough: bool,

        /// Copy Claude Code session data from source worktree path
        #[arg(long)]
        copy_session_from: Option<String>,
    },

    /// Clean up a review environment
    Cleanup {
        /// Pull request number to clean up
        #[arg(short, long)]
        pr: u32,

        /// Skip confirmation prompt (--force/--yes)
        #[arg(short, long, alias = "yes")]
        force: bool,
    },

    /// List active review environments
    List,

    /// Show status of a review environment
    Status {
        /// Pull request number
        #[arg(short, long)]
        pr: u32,
    },

    /// Initialize configuration
    Config {
        /// Initialize local config in current directory
        #[arg(short, long)]
        local: bool,
    },

    /// View AI agent analysis results
    AgentResult {
        /// Pull request number
        #[arg(short, long)]
        pr: u32,
    },

    /// Merge a branch into the worktree
    Merge {
        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Branch to merge from
        #[arg(short, long)]
        from: String,
    },

    /// Rebase the worktree onto another branch
    Rebase {
        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Branch to rebase onto
        #[arg(short, long)]
        onto: String,
    },

    /// Launch TUI (Terminal User Interface)
    Tui,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    let result = match cli.command {
        Commands::Review {
            pr,
            branch,
            force,
            worktree,
            with_agent,
            thorough,
            copy_session_from,
        } => commands::review::execute(pr, branch, force, worktree, with_agent, thorough, copy_session_from).await,
        Commands::Cleanup { pr, force } => commands::cleanup::execute(pr, force).await,
        Commands::List => commands::list::execute().await,
        Commands::Status { pr } => commands::status::execute(pr).await,
        Commands::Config { local } => commands::config::execute(local).await,
        Commands::AgentResult { pr } => commands::agent_result::execute(pr).await,
        Commands::Merge { pr, from } => commands::merge::execute(pr, from).await,
        Commands::Rebase { pr, onto } => commands::rebase::execute(pr, onto).await,
        Commands::Tui => commands::tui::execute().await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
