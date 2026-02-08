// Library exports for testing and potential future library usage

pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;

// Re-export commonly used types
pub use config::Config;
pub use error::{ChabaError, Result};
