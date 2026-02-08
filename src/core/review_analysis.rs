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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let finding = Finding::new(
            Severity::High,
            Category::Security,
            "SQL Injection vulnerability".to_string(),
            "User input is not sanitized".to_string(),
        );

        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.category, Category::Security);
        assert_eq!(finding.title, "SQL Injection vulnerability");
        assert_eq!(finding.description, "User input is not sanitized");
        assert!(finding.file.is_none());
        assert!(finding.line.is_none());
        assert!(finding.suggestion.is_none());
    }

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new(
            Severity::Medium,
            Category::Performance,
            "Inefficient loop".to_string(),
            "Consider using iterator".to_string(),
        )
        .with_file("src/main.rs".to_string())
        .with_line(42)
        .with_suggestion("Use .iter().filter() instead".to_string());

        assert_eq!(finding.file, Some("src/main.rs".to_string()));
        assert_eq!(finding.line, Some(42));
        assert_eq!(
            finding.suggestion,
            Some("Use .iter().filter() instead".to_string())
        );
    }

    #[test]
    fn test_review_analysis_creation() {
        let analysis = ReviewAnalysis::new("claude".to_string());

        assert_eq!(analysis.agent, "claude");
        assert!(analysis.score.is_none());
        assert!(analysis.findings.is_empty());
        assert!(analysis.raw_output.is_none());
        assert!(!analysis.timestamp.is_empty());
    }

    #[test]
    fn test_review_analysis_add_finding() {
        let mut analysis = ReviewAnalysis::new("codex".to_string());

        let finding = Finding::new(
            Severity::Critical,
            Category::Security,
            "Hardcoded credentials".to_string(),
            "API key found in source code".to_string(),
        );

        analysis.add_finding(finding);

        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_review_analysis_set_score() {
        let mut analysis = ReviewAnalysis::new("gemini".to_string());

        analysis.set_score(4.5);
        assert_eq!(analysis.score, Some(4.5));

        // Test clamping
        analysis.set_score(6.0);
        assert_eq!(analysis.score, Some(5.0));

        analysis.set_score(-1.0);
        assert_eq!(analysis.score, Some(0.0));
    }

    #[test]
    fn test_count_by_severity() {
        let mut analysis = ReviewAnalysis::new("claude".to_string());

        analysis.add_finding(Finding::new(
            Severity::Critical,
            Category::Security,
            "Issue 1".to_string(),
            "Description 1".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::Critical,
            Category::Security,
            "Issue 2".to_string(),
            "Description 2".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::High,
            Category::Performance,
            "Issue 3".to_string(),
            "Description 3".to_string(),
        ));

        assert_eq!(analysis.count_by_severity(&Severity::Critical), 2);
        assert_eq!(analysis.count_by_severity(&Severity::High), 1);
        assert_eq!(analysis.count_by_severity(&Severity::Medium), 0);
    }

    #[test]
    fn test_count_by_category() {
        let mut analysis = ReviewAnalysis::new("codex".to_string());

        analysis.add_finding(Finding::new(
            Severity::High,
            Category::Security,
            "Issue 1".to_string(),
            "Description 1".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::Medium,
            Category::Security,
            "Issue 2".to_string(),
            "Description 2".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::Low,
            Category::Performance,
            "Issue 3".to_string(),
            "Description 3".to_string(),
        ));

        assert_eq!(analysis.count_by_category(&Category::Security), 2);
        assert_eq!(analysis.count_by_category(&Category::Performance), 1);
        assert_eq!(analysis.count_by_category(&Category::BestPractice), 0);
    }

    #[test]
    fn test_critical_findings() {
        let mut analysis = ReviewAnalysis::new("gemini".to_string());

        analysis.add_finding(Finding::new(
            Severity::Critical,
            Category::Security,
            "Critical issue".to_string(),
            "Description".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::High,
            Category::Security,
            "High issue".to_string(),
            "Description".to_string(),
        ));

        analysis.add_finding(Finding::new(
            Severity::Medium,
            Category::BestPractice,
            "Medium issue".to_string(),
            "Description".to_string(),
        ));

        let critical = analysis.critical_findings();
        assert_eq!(critical.len(), 2);
        assert!(matches!(critical[0].severity, Severity::Critical));
        assert!(matches!(critical[1].severity, Severity::High));
    }

    #[test]
    fn test_severity_serialization() {
        let critical = Severity::Critical;
        let json = serde_json::to_string(&critical).unwrap();
        assert_eq!(json, "\"critical\"");

        let high = Severity::High;
        let json = serde_json::to_string(&high).unwrap();
        assert_eq!(json, "\"high\"");
    }

    #[test]
    fn test_category_serialization() {
        let security = Category::Security;
        let json = serde_json::to_string(&security).unwrap();
        assert_eq!(json, "\"security\"");

        let best_practice = Category::BestPractice;
        let json = serde_json::to_string(&best_practice).unwrap();
        assert_eq!(json, "\"best-practice\"");
    }

    #[test]
    fn test_finding_serialization() {
        let finding = Finding::new(
            Severity::High,
            Category::Security,
            "Test finding".to_string(),
            "Test description".to_string(),
        )
        .with_file("test.rs".to_string())
        .with_line(100);

        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("\"severity\":\"high\""));
        assert!(json.contains("\"category\":\"security\""));
        assert!(json.contains("\"title\":\"Test finding\""));
        assert!(json.contains("\"file\":\"test.rs\""));
        assert!(json.contains("\"line\":100"));
    }

    #[test]
    fn test_review_analysis_serialization() {
        let mut analysis = ReviewAnalysis::new("claude".to_string());
        analysis.set_score(4.0);
        analysis.add_finding(Finding::new(
            Severity::Medium,
            Category::CodeQuality,
            "Code smell".to_string(),
            "Refactor needed".to_string(),
        ));

        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"agent\":\"claude\""));
        assert!(json.contains("\"score\":4.0"));
        assert!(json.contains("\"findings\""));
    }
}
