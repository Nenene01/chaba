use std::collections::HashSet;
use std::net::TcpListener;

use crate::core::state::State;
use crate::error::{ChabaError, Result};

pub struct PortManager {
    range_start: u16,
    range_end: u16,
}

impl PortManager {
    pub fn new(range_start: u16, range_end: u16) -> Self {
        Self {
            range_start,
            range_end,
        }
    }

    /// Assign an available port
    pub fn assign_port(&self, state: &State) -> Result<u16> {
        // Collect already assigned ports
        let used_ports: HashSet<u16> = state
            .reviews
            .iter()
            .filter_map(|r| r.port)
            .collect();

        // Find an available port
        for port in self.range_start..=self.range_end {
            if !used_ports.contains(&port) && !is_port_in_use(port) {
                return Ok(port);
            }
        }

        Err(ChabaError::NoAvailablePort {
            range_start: self.range_start,
            range_end: self.range_end,
        })
    }
}

/// Check if a port is currently in use
fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::ReviewState;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_assign_port() {
        let manager = PortManager::new(3000, 3010);
        let state = State::default();

        let port = manager.assign_port(&state).unwrap();
        assert!(port >= 3000 && port <= 3010);
    }

    #[test]
    fn test_avoid_used_ports() {
        let manager = PortManager::new(3000, 3002);

        let mut state = State::default();
        state.reviews.push(ReviewState {
            pr_number: 1,
            branch: "test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: None,
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        });

        let port = manager.assign_port(&state).unwrap();
        assert_ne!(port, 3000);
        assert!(port >= 3000 && port <= 3002);
    }

    #[test]
    fn test_no_available_port() {
        let manager = PortManager::new(3000, 3000);

        let mut state = State::default();
        state.reviews.push(ReviewState {
            pr_number: 1,
            branch: "test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: None,
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        });

        let result = manager.assign_port(&state);
        assert!(result.is_err());
    }
}
