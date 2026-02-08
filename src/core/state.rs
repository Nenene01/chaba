use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::core::review_analysis::ReviewAnalysis;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    pub pr_number: u32,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub created_at: DateTime<Utc>,

    // Phase 2: Sandbox environment info
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,

    #[serde(default)]
    pub deps_installed: bool,

    #[serde(default)]
    pub env_copied: bool,

    // Phase 3: AI Agent analysis results
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_analyses: Vec<ReviewAnalysis>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    /// State version for optimistic locking
    /// Incremented on every save to detect concurrent modifications
    #[serde(default)]
    pub version: u64,

    pub reviews: Vec<ReviewState>,
}

impl State {
    /// Load state from file with shared lock
    pub fn load() -> Result<Self> {
        let state_path = Self::state_file_path()?;

        if !state_path.exists() {
            return Ok(State::default());
        }

        // Open file with shared lock for reading
        let file = File::open(&state_path)?;
        file.lock_shared()?;

        let content = std::fs::read_to_string(&state_path)?;
        let state: State = serde_yaml::from_str(&content)?;

        // Lock is automatically released when file is dropped
        Ok(state)
    }

    /// Save state to file with atomic write and optimistic locking
    pub fn save(&mut self) -> Result<()> {
        let state_path = Self::state_file_path()?;

        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Optimistic locking: Check if file was modified by another process
        if state_path.exists() {
            // Read current version from file
            let file = File::open(&state_path)?;
            file.lock_shared()?;

            let content = std::fs::read_to_string(&state_path)?;
            if let Ok(current_state) = serde_yaml::from_str::<State>(&content) {
                if current_state.version != self.version {
                    return Err(crate::error::ChabaError::StateConflict {
                        expected: self.version,
                        actual: current_state.version,
                    });
                }
            }
            // Lock is released when file is dropped
        }

        // Increment version before saving
        self.version += 1;

        let content = serde_yaml::to_string(&self)?;

        // Use tempfile + rename for atomic write
        // Create temp file in same directory as target to ensure same filesystem
        let temp_file = NamedTempFile::new_in(
            state_path.parent().expect("state path should have parent directory")
        )?;

        // Lock the temp file exclusively
        temp_file.as_file().lock_exclusive()?;

        // Write to temp file
        std::fs::write(temp_file.path(), &content)?;

        // Set file permissions to 600 (rw-------) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(temp_file.path())?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(temp_file.path(), perms)?;
        }

        // Atomic rename (replaces existing file)
        // persist() returns PersistError which contains the underlying io::Error
        temp_file.persist(&state_path)
            .map_err(|e| e.error)?;

        // Lock is automatically released when temp_file is dropped
        Ok(())
    }

    /// Add a review to state
    pub fn add_review(&mut self, review: ReviewState) -> Result<()> {
        // Remove existing review with same PR number
        self.reviews.retain(|r| r.pr_number != review.pr_number);
        self.reviews.push(review);
        self.save()
    }

    /// Remove a review from state
    pub fn remove_review(&mut self, pr_number: u32) -> Result<()> {
        self.reviews.retain(|r| r.pr_number != pr_number);
        self.save()
    }

    /// Get review by PR number
    pub fn get_review(&self, pr_number: u32) -> Option<&ReviewState> {
        self.reviews.iter().find(|r| r.pr_number == pr_number)
    }

    /// Get state file path
    fn state_file_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::ChabaError::ConfigError("Cannot find home directory".to_string())
        })?;

        Ok(home.join(".chaba").join("state.yaml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::review_analysis::{Finding, ReviewAnalysis, Severity, Category};

    #[test]
    fn test_state_default() {
        let state = State::default();
        assert!(state.reviews.is_empty());
    }

    #[test]
    fn test_review_state_creation() {
        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: Vec::new(),
        };

        assert_eq!(review.pr_number, 123);
        assert_eq!(review.branch, "feature/test");
        assert_eq!(review.port, Some(3000));
        assert!(review.deps_installed);
        assert!(review.env_copied);
        assert!(review.agent_analyses.is_empty());
    }

    #[test]
    fn test_state_add_review() {
        let mut state = State::default();

        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: Vec::new(),
        };

        state.reviews.push(review);
        assert_eq!(state.reviews.len(), 1);
        assert_eq!(state.reviews[0].pr_number, 123);
    }

    #[test]
    fn test_state_get_review() {
        let mut state = State::default();

        let review1 = ReviewState {
            pr_number: 123,
            branch: "feature/test1".to_string(),
            worktree_path: PathBuf::from("/tmp/test1"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: Vec::new(),
        };

        let review2 = ReviewState {
            pr_number: 456,
            branch: "feature/test2".to_string(),
            worktree_path: PathBuf::from("/tmp/test2"),
            created_at: Utc::now(),
            port: Some(3001),
            project_type: Some("rust".to_string()),
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        };

        state.reviews.push(review1);
        state.reviews.push(review2);

        let found = state.get_review(123);
        assert!(found.is_some());
        assert_eq!(found.unwrap().branch, "feature/test1");

        let not_found = state.get_review(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_state_remove_review() {
        let mut state = State::default();

        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: None,
            project_type: None,
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        };

        state.reviews.push(review);
        assert_eq!(state.reviews.len(), 1);

        state.reviews.retain(|r| r.pr_number != 123);
        assert_eq!(state.reviews.len(), 0);
    }

    #[test]
    fn test_review_state_with_agent_analyses() {
        let mut analysis = ReviewAnalysis::new("claude".to_string());
        analysis.add_finding(Finding::new(
            Severity::High,
            Category::Security,
            "Security issue".to_string(),
            "Fix this vulnerability".to_string(),
        ));

        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: vec![analysis],
        };

        assert_eq!(review.agent_analyses.len(), 1);
        assert_eq!(review.agent_analyses[0].agent, "claude");
        assert_eq!(review.agent_analyses[0].findings.len(), 1);
    }

    #[test]
    fn test_state_serialization() {
        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: Vec::new(),
        };

        let state = State {
            version: 0,
            reviews: vec![review],
        };

        let yaml = serde_yaml::to_string(&state).unwrap();
        assert!(yaml.contains("pr_number: 123"));
        assert!(yaml.contains("branch: feature/test"));
        assert!(yaml.contains("port: 3000"));
        assert!(yaml.contains("project_type: node"));
    }

    #[test]
    fn test_state_deserialization() {
        let yaml = r#"
reviews:
  - pr_number: 123
    branch: feature/test
    worktree_path: /tmp/test
    created_at: 2024-01-01T00:00:00Z
    port: 3000
    project_type: node
    deps_installed: true
    env_copied: true
"#;

        let state: State = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(state.reviews.len(), 1);
        assert_eq!(state.reviews[0].pr_number, 123);
        assert_eq!(state.reviews[0].branch, "feature/test");
        assert_eq!(state.reviews[0].port, Some(3000));
        assert_eq!(state.reviews[0].project_type, Some("node".to_string()));
        assert!(state.reviews[0].deps_installed);
        assert!(state.reviews[0].env_copied);
        assert!(state.reviews[0].agent_analyses.is_empty());
    }

    #[test]
    fn test_backward_compatibility_without_phase2_fields() {
        // Old format without Phase 2 fields
        let yaml = r#"
reviews:
  - pr_number: 123
    branch: feature/test
    worktree_path: /tmp/test
    created_at: 2024-01-01T00:00:00Z
"#;

        let state: State = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(state.reviews.len(), 1);
        assert_eq!(state.reviews[0].pr_number, 123);
        assert_eq!(state.reviews[0].port, None);
        assert_eq!(state.reviews[0].project_type, None);
        assert!(!state.reviews[0].deps_installed);
        assert!(!state.reviews[0].env_copied);
        assert!(state.reviews[0].agent_analyses.is_empty());
    }

    #[test]
    fn test_state_serialization_skips_empty_agent_analyses() {
        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: Vec::new(),
        };

        let state = State {
            version: 0,
            reviews: vec![review],
        };

        let yaml = serde_yaml::to_string(&state).unwrap();
        // Should not contain agent_analyses field when empty
        assert!(!yaml.contains("agent_analyses"));
    }

    #[test]
    fn test_state_serialization_includes_agent_analyses() {
        let mut analysis = ReviewAnalysis::new("claude".to_string());
        analysis.add_finding(Finding::new(
            Severity::High,
            Category::Security,
            "Issue".to_string(),
            "Description".to_string(),
        ));

        let review = ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: Some("node".to_string()),
            deps_installed: true,
            env_copied: true,
            agent_analyses: vec![analysis],
        };

        let state = State {
            version: 0,
            reviews: vec![review],
        };

        let yaml = serde_yaml::to_string(&state).unwrap();
        // Should contain agent_analyses when not empty
        assert!(yaml.contains("agent_analyses"));
    }

    #[test]
    fn test_version_increments_on_save() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut state = State::default();
        assert_eq!(state.version, 0);

        // First save
        state.save().unwrap();
        assert_eq!(state.version, 1);

        // Second save
        state.save().unwrap();
        assert_eq!(state.version, 2);

        // Load and verify version
        let loaded = State::load().unwrap();
        assert_eq!(loaded.version, 2);
    }

    #[test]
    fn test_concurrent_modification_detection() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        // Ensure .chaba directory exists
        let chaba_dir = temp_dir.path().join(".chaba");
        std::fs::create_dir_all(&chaba_dir).unwrap();

        // Create initial state
        let mut state1 = State::default();
        state1.save().unwrap();
        assert_eq!(state1.version, 1);

        // Simulate two processes loading the same state
        let mut state2 = State::load().unwrap();
        let mut state3 = State::load().unwrap();
        assert_eq!(state2.version, 1);
        assert_eq!(state3.version, 1);

        // Process 2 modifies and saves
        state2.reviews.push(ReviewState {
            pr_number: 123,
            branch: "feature/test".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            created_at: Utc::now(),
            port: Some(3000),
            project_type: None,
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        });
        state2.save().unwrap();
        assert_eq!(state2.version, 2);

        // Process 3 tries to save - should fail due to conflict
        state3.reviews.push(ReviewState {
            pr_number: 456,
            branch: "feature/other".to_string(),
            worktree_path: PathBuf::from("/tmp/other"),
            created_at: Utc::now(),
            port: Some(3001),
            project_type: None,
            deps_installed: false,
            env_copied: false,
            agent_analyses: Vec::new(),
        });

        let result = state3.save();
        assert!(result.is_err());

        match result {
            Err(crate::error::ChabaError::StateConflict { expected, actual }) => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected StateConflict error"),
        }
    }

    #[test]
    fn test_version_backward_compatibility() {
        // Old format without version field
        let yaml = r#"
reviews:
  - pr_number: 123
    branch: feature/test
    worktree_path: /tmp/test
    created_at: 2024-01-01T00:00:00Z
"#;

        let state: State = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(state.version, 0); // Default value
        assert_eq!(state.reviews.len(), 1);
    }
}
