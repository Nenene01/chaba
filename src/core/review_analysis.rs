use serde::{Deserialize, Serialize};

/// Severity level of a finding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Category of a finding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Security,
    Performance,
    BestPractice,
    CodeQuality,
    Architecture,
    Testing,
    Documentation,
    Other,
}

/// Individual finding from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Severity level
    pub severity: Severity,

    /// Category
    pub category: Category,

    /// File path (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Line number (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Brief title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Suggested fix or improvement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Analysis result from a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnalysis {
    /// Agent name (claude, codex, gemini)
    pub agent: String,

    /// Analysis timestamp (ISO 8601)
    pub timestamp: String,

    /// Overall score (0.0 - 5.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,

    /// List of findings
    pub findings: Vec<Finding>,

    /// Raw output from agent (fallback)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_output: Option<String>,
}

impl ReviewAnalysis {
    /// Create a new ReviewAnalysis
    pub fn new(agent: String) -> Self {
        ReviewAnalysis {
            agent,
            timestamp: chrono::Utc::now().to_rfc3339(),
            score: None,
            findings: Vec::new(),
            raw_output: None,
        }
    }

    /// Add a finding
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Set overall score
    #[allow(dead_code)]
    pub fn set_score(&mut self, score: f32) {
        self.score = Some(score.clamp(0.0, 5.0));
    }

    /// Set raw output as fallback
    pub fn set_raw_output(&mut self, output: String) {
        self.raw_output = Some(output);
    }

    /// Count findings by severity
    pub fn count_by_severity(&self, severity: &Severity) -> usize {
        self.findings.iter().filter(|f| &f.severity == severity).count()
    }

    /// Count findings by category
    pub fn count_by_category(&self, category: &Category) -> usize {
        self.findings.iter().filter(|f| &f.category == category).count()
    }

    /// Get critical and high severity findings
    #[allow(dead_code)]
    pub fn critical_findings(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .collect()
    }
}

impl Finding {
    /// Create a new finding
    pub fn new(
        severity: Severity,
        category: Category,
        title: String,
        description: String,
    ) -> Self {
        Finding {
            severity,
            category,
            file: None,
            line: None,
            title,
            description,
            suggestion: None,
        }
    }

    /// Set file location
    #[allow(dead_code)]
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    /// Set line number
    #[allow(dead_code)]
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set suggestion
    #[allow(dead_code)]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}
