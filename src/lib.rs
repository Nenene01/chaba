//! Chaba (茶葉) - AI Agent Friendly Source Review & Debug Environment
//!
//! Chaba integrates git worktree, branch operations, and sandbox environments
//! for seamless team collaboration with AI agents.
//!
//! # Features
//!
//! - **Git Worktree Management**: Create isolated review environments for PRs
//! - **Automatic Sandbox Setup**: Detect project type and install dependencies
//! - **AI Agent Integration**: Automated code review with multiple AI agents
//! - **Port Management**: Automatic port assignment for parallel development
//! - **Environment Copying**: Copy .env files to review environments
//!
//! # Example
//!
//! ```rust,no_run
//! use chaba::Config;
//!
//! // Load configuration
//! let config = Config::load().unwrap();
//!
//! // Access configuration
//! println!("Port range: {}-{}",
//!     config.sandbox.port.range_start,
//!     config.sandbox.port.range_end
//! );
//! ```
//!
//! # CLI Usage
//!
//! ```bash
//! # Create review environment for PR #123
//! chaba review --pr 123
//!
//! # Create review with AI agent analysis
//! chaba review --pr 123 --with-agent
//!
//! # Thorough review with all agents
//! chaba review --pr 123 --thorough
//!
//! # View agent analysis results
//! chaba agent-result --pr 123
//!
//! # List active reviews
//! chaba list
//!
//! # Clean up review environment
//! chaba cleanup --pr 123
//! ```

pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;

// Re-export commonly used types
pub use config::Config;
pub use error::{ChabaError, Result};
