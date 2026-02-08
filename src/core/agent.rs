use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::config::AgentsConfig;
use crate::core::command::{CommandRunner, LiveCommandRunner};
use crate::core::review_analysis::{ReviewAnalysis, Finding, Severity, Category};
use crate::error::{ChabaError, Result};

pub struct AgentManager {
    config: AgentsConfig,
    runner: Arc<dyn CommandRunner + Send + Sync>,
}

impl AgentManager {
    /// Create a new AgentManager with custom command runner
    ///
    /// This constructor is primarily for testing, allowing injection of a mock runner.
    pub fn new_with_runner(
        config: AgentsConfig,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Self {
        AgentManager { config, runner }
    }

    /// Create a new AgentManager with default LiveCommandRunner
    pub fn new(config: AgentsConfig) -> Self {
        Self::new_with_runner(config, Arc::new(LiveCommandRunner))
    }

    /// Run agents for PR review
    pub async fn run_review(
        &self,
        pr_number: u32,
        worktree_path: &Path,
        thorough: bool,
    ) -> Result<Vec<ReviewAnalysis>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let agents = if thorough {
            &self.config.thorough_agents
        } else {
            &self.config.default_agents
        };

        if self.config.parallel {
            self.run_parallel(agents, pr_number, worktree_path).await
        } else {
            self.run_sequential(agents, pr_number, worktree_path).await
        }
    }

    /// Run agents in parallel
    async fn run_parallel(
        &self,
        agents: &[String],
        pr_number: u32,
        worktree_path: &Path,
    ) -> Result<Vec<ReviewAnalysis>> {
        // Create progress bar
        let pb = ProgressBar::new(agents.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message("Running AI agents");

        let mut tasks = Vec::new();

        for agent in agents {
            let agent = agent.clone();
            let worktree_path = worktree_path.to_path_buf();
            let timeout = self.config.timeout;
            let runner = self.runner.clone();

            tasks.push(tokio::spawn(async move {
                Self::run_single_agent(&agent, pr_number, &worktree_path, timeout, runner).await
            }));
        }

        let results = futures::future::join_all(tasks).await;

        let mut analyses = Vec::new();
        let mut errors = Vec::new();

        for (idx, result) in results.into_iter().enumerate() {
            let agent_name = &agents[idx];
            match result {
                Ok(Ok(analysis)) => {
                    pb.set_message(format!("✓ {} completed", agent_name));
                    tracing::info!("✓ {} completed analysis", agent_name);
                    analyses.push(analysis);
                }
                Ok(Err(e)) => {
                    pb.set_message(format!("✗ {} failed", agent_name));
                    tracing::warn!("✗ {} failed: {}", agent_name, e);
                    errors.push((agent_name.clone(), e.to_string()));
                }
                Err(e) => {
                    pb.set_message(format!("✗ {} task failed", agent_name));
                    tracing::warn!("✗ {} task failed: {}", agent_name, e);
                    errors.push((agent_name.clone(), e.to_string()));
                }
            }
            pb.inc(1);
        }

        if !errors.is_empty() && analyses.is_empty() {
            // All agents failed
            pb.finish_with_message("⚠️  All agents failed");
            tracing::error!("⚠️  All agents failed to complete analysis");
            tracing::error!("Review the errors above and check:");
            tracing::error!("  - Agent CLI tools are installed (claude, codex, gemini)");
            tracing::error!("  - Network connectivity");
            tracing::error!("  - Agent timeout setting (current: {}s)", self.config.timeout);
        } else if !errors.is_empty() {
            // Some agents failed
            pb.finish_with_message(format!("{} agents completed, {} failed", analyses.len(), errors.len()));
            tracing::warn!("⚠️  {} agent(s) failed, {} succeeded", errors.len(), analyses.len());
        } else {
            // All succeeded
            pb.finish_with_message("✓ All agents completed successfully");
        }

        Ok(analyses)
    }

    /// Run agents sequentially
    async fn run_sequential(
        &self,
        agents: &[String],
        pr_number: u32,
        worktree_path: &Path,
    ) -> Result<Vec<ReviewAnalysis>> {
        // Create progress bar
        let pb = ProgressBar::new(agents.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );

        let mut analyses = Vec::new();
        let mut errors = Vec::new();

        for agent in agents {
            pb.set_message(format!("Running {} analysis...", agent));
            tracing::info!("Running {} analysis...", agent);
            match Self::run_single_agent(agent, pr_number, worktree_path, self.config.timeout, self.runner.clone()).await {
                Ok(analysis) => {
                    pb.set_message(format!("✓ {} completed", agent));
                    tracing::info!("✓ {} completed", agent);
                    analyses.push(analysis);
                }
                Err(e) => {
                    pb.set_message(format!("✗ {} failed", agent));
                    tracing::warn!("✗ {} failed: {}", agent, e);
                    errors.push((agent.clone(), e.to_string()));
                }
            }
            pb.inc(1);
        }

        if !errors.is_empty() && analyses.is_empty() {
            pb.finish_with_message("⚠️  All agents failed");
            tracing::error!("⚠️  All agents failed to complete analysis");
            tracing::error!("Check agent CLI tool installations and network connectivity");
        } else if !errors.is_empty() {
            pb.finish_with_message(format!("{} agents completed, {} failed", analyses.len(), errors.len()));
            tracing::warn!("⚠️  {} agent(s) failed, {} succeeded", errors.len(), analyses.len());
        } else {
            pb.finish_with_message("✓ All agents completed successfully");
        }

        Ok(analyses)
    }

    /// Run a single agent with timeout
    async fn run_single_agent(
        agent: &str,
        pr_number: u32,
        worktree_path: &Path,
        timeout_secs: u64,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Result<ReviewAnalysis> {
        let timeout = Duration::from_secs(timeout_secs);

        let result = tokio::time::timeout(
            timeout,
            Self::execute_agent(agent, pr_number, worktree_path, runner),
        )
        .await;

        match result {
            Ok(Ok(analysis)) => Ok(analysis),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ChabaError::Other(anyhow::anyhow!(
                "Agent {} timed out after {} seconds",
                agent,
                timeout_secs
            ))),
        }
    }

    /// Execute a specific agent
    async fn execute_agent(
        agent: &str,
        pr_number: u32,
        worktree_path: &Path,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Result<ReviewAnalysis> {
        let mut analysis = ReviewAnalysis::new(agent.to_string());

        match agent {
            "claude" => Self::run_claude(pr_number, worktree_path, &mut analysis, runner).await?,
            "codex" => Self::run_codex(pr_number, worktree_path, &mut analysis, runner).await?,
            "gemini" => Self::run_gemini(pr_number, worktree_path, &mut analysis, runner).await?,
            _ => {
                return Err(ChabaError::ConfigError(format!(
                    "Unknown agent: {}",
                    agent
                )))
            }
        }

        Ok(analysis)
    }

    /// Run Claude Code agent
    async fn run_claude(
        pr_number: u32,
        worktree_path: &Path,
        analysis: &mut ReviewAnalysis,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Result<()> {
        let prompt = format!(
            "PR #{} のコードレビューを実施してください。品質、セキュリティ、パフォーマンスの観点から分析し、改善点を指摘してください。",
            pr_number
        );

        let output = runner
            .run(
                "claude",
                &[
                    "--model".as_ref(),
                    "sonnet".as_ref(),
                    "--yes".as_ref(),
                    OsStr::new(&prompt),
                ],
                worktree_path,
            )
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ChabaError::AgentExecutionError {
                agent: "claude".to_string(),
                stdout,
                stderr,
            })
        }
    }

    /// Run Codex agent
    async fn run_codex(
        pr_number: u32,
        worktree_path: &Path,
        analysis: &mut ReviewAnalysis,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Result<()> {
        let prompt = format!(
            "このPR #{}のコードをレビューしてください。バグ、セキュリティ問題、ベストプラクティス違反を指摘してください。",
            pr_number
        );

        let output = runner
            .run(
                "codex",
                &[
                    "exec".as_ref(),
                    "--full-auto".as_ref(),
                    "--sandbox".as_ref(),
                    "read-only".as_ref(),
                    OsStr::new(&prompt),
                ],
                worktree_path,
            )
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ChabaError::AgentExecutionError {
                agent: "codex".to_string(),
                stdout,
                stderr,
            })
        }
    }

    /// Run Gemini agent
    async fn run_gemini(
        pr_number: u32,
        worktree_path: &Path,
        analysis: &mut ReviewAnalysis,
        runner: Arc<dyn CommandRunner + Send + Sync>,
    ) -> Result<()> {
        let prompt = format!(
            "このPR #{}を戦略的視点からレビューしてください。アーキテクチャ、設計パターン、拡張性について分析してください。",
            pr_number
        );

        let output = runner
            .run(
                "gemini",
                &[
                    "-m".as_ref(),
                    "gemini-2.5-pro".as_ref(),
                    "-s".as_ref(),
                    "-y".as_ref(),
                    "-p".as_ref(),
                    OsStr::new(&prompt),
                ],
                worktree_path,
            )
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ChabaError::AgentExecutionError {
                agent: "gemini".to_string(),
                stdout,
                stderr,
            })
        }
    }

    /// Parse agent output and extract findings
    ///
    /// This function attempts to parse the output in the following order:
    /// 1. JSON format (structured output from agents)
    /// 2. Enhanced pattern matching (keywords and severity indicators)
    /// 3. Fallback to basic info finding
    fn parse_output(output: &str, analysis: &mut ReviewAnalysis) {
        // Store raw output as fallback
        analysis.set_raw_output(output.to_string());

        // Try JSON parsing first
        if Self::try_parse_json(output, analysis) {
            return;
        }

        // Enhanced pattern matching with more keywords
        Self::parse_with_patterns(output, analysis);

        // If no structured findings were extracted, create a general info finding
        if analysis.findings.is_empty() {
            let finding = Finding::new(
                Severity::Info,
                Category::Other,
                "Review completed".to_string(),
                "Agent completed review - see raw output for details".to_string(),
            );
            analysis.add_finding(finding);
        }
    }

    /// Try to parse output as JSON
    fn try_parse_json(output: &str, analysis: &mut ReviewAnalysis) -> bool {
        use serde_json::Value;

        // Try to find JSON object or array in the output
        // Look for JSON between common delimiters
        let json_str = if let Some(start) = output.find('{') {
            &output[start..]
        } else if let Some(start) = output.find('[') {
            &output[start..]
        } else {
            return false;
        };

        // Try to parse as JSON
        let parsed: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => {
                // Try to extract JSON more carefully
                for line in output.lines() {
                    if line.trim().starts_with('{') || line.trim().starts_with('[') {
                        if let Ok(v) = serde_json::from_str(line.trim()) {
                            v
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                return false;
            }
        };

        // Extract findings from JSON
        let findings = if let Some(findings_array) = parsed.get("findings").and_then(|v| v.as_array()) {
            findings_array
        } else if parsed.is_array() {
            parsed.as_array().unwrap()
        } else {
            return false;
        };

        for finding_value in findings {
            if let Some(finding) = Self::parse_json_finding(finding_value) {
                analysis.add_finding(finding);
            }
        }

        // Extract score if present
        if let Some(score) = parsed.get("score").and_then(|v| v.as_f64()) {
            analysis.set_score(score as f32);
        }

        !analysis.findings.is_empty()
    }

    /// Parse a single finding from JSON value
    fn parse_json_finding(value: &serde_json::Value) -> Option<Finding> {
        let severity_str = value.get("severity")?.as_str()?;
        let severity = match severity_str.to_lowercase().as_str() {
            "critical" | "重大" => Severity::Critical,
            "high" | "高" => Severity::High,
            "medium" | "中" => Severity::Medium,
            "low" | "低" => Severity::Low,
            _ => Severity::Info,
        };

        let category_str = value.get("category").and_then(|v| v.as_str()).unwrap_or("other");
        let category = match category_str.to_lowercase().as_str() {
            "security" | "セキュリティ" => Category::Security,
            "performance" | "パフォーマンス" => Category::Performance,
            "bug" | "バグ" | "codequality" | "code_quality" => Category::CodeQuality,
            "bestpractice" | "best_practice" | "ベストプラクティス" => Category::BestPractice,
            "architecture" | "アーキテクチャ" => Category::Architecture,
            "testing" | "テスト" => Category::Testing,
            "documentation" | "ドキュメント" => Category::Documentation,
            _ => Category::Other,
        };

        let title = value.get("title")?.as_str()?.to_string();
        let description = value.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut finding = Finding::new(severity, category, title, description);

        // Optional fields
        if let Some(file) = value.get("file").and_then(|v| v.as_str()) {
            finding = finding.with_file(file.to_string());
        }
        if let Some(line) = value.get("line").and_then(|v| v.as_u64()) {
            finding = finding.with_line(line as u32);
        }
        if let Some(suggestion) = value.get("suggestion").and_then(|v| v.as_str()) {
            finding = finding.with_suggestion(suggestion.to_string());
        }

        Some(finding)
    }

    /// Enhanced pattern matching for text output
    fn parse_with_patterns(output: &str, analysis: &mut ReviewAnalysis) {
        let lines: Vec<&str> = output.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();

            // Determine severity based on keywords
            let (severity, category) = if line_lower.contains("critical")
                || line_lower.contains("重大")
                || line_lower.contains("致命的") {
                (Severity::Critical, Category::Security)
            } else if line_lower.contains("security")
                || line_lower.contains("セキュリティ")
                || line_lower.contains("vulnerability")
                || line_lower.contains("脆弱性") {
                (Severity::High, Category::Security)
            } else if line_lower.contains("error")
                || line_lower.contains("エラー")
                || line_lower.contains("bug")
                || line_lower.contains("バグ") {
                (Severity::High, Category::CodeQuality)
            } else if line_lower.contains("warning")
                || line_lower.contains("警告") {
                (Severity::Medium, Category::BestPractice)
            } else if line_lower.contains("performance")
                || line_lower.contains("パフォーマンス")
                || line_lower.contains("slow")
                || line_lower.contains("遅い") {
                (Severity::Medium, Category::Performance)
            } else if line_lower.contains("suggestion")
                || line_lower.contains("提案")
                || line_lower.contains("improvement")
                || line_lower.contains("改善") {
                (Severity::Low, Category::BestPractice)
            } else {
                continue;
            };

            let title = line.trim().to_string();
            let description = lines.get(i + 1).unwrap_or(&"").trim().to_string();

            let finding = Finding::new(severity, category, title, description);
            analysis.add_finding(finding);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};
    use std::sync::Mutex;

    // Simple mock implementation for testing
    struct TestCommandRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>, // (program, args)
        return_output: Output,
    }

    impl TestCommandRunner {
        fn new(output: Output) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                return_output: output,
            }
        }

        fn get_calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl CommandRunner for TestCommandRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&OsStr],
            _current_dir: &Path,
        ) -> std::result::Result<Output, std::io::Error> {
            let mut calls = self.calls.lock().unwrap();
            calls.push((
                program.to_string(),
                args.iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect(),
            ));
            Ok(self.return_output.clone())
        }
    }

    // Helper to create a successful output
    fn success_output(stdout: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: vec![],
        }
    }

    // Helper to create a failed output
    fn error_output(stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[tokio::test]
    async fn test_parse_output_json() {
        let json_output = r#"
        {
            "findings": [
                {
                    "severity": "high",
                    "category": "security",
                    "title": "SQL Injection vulnerability",
                    "description": "User input not sanitized"
                }
            ],
            "score": 4.2
        }
        "#;

        let mut analysis = ReviewAnalysis::new("test".to_string());
        AgentManager::parse_output(json_output, &mut analysis);

        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].severity, Severity::High);
        assert_eq!(analysis.findings[0].category, Category::Security);
        assert!(analysis.score.is_some());
        assert_eq!(analysis.score.unwrap(), 4.2);
    }

    #[tokio::test]
    async fn test_parse_output_pattern_matching() {
        let text_output = "Critical: Security vulnerability found\nThis is a serious issue";

        let mut analysis = ReviewAnalysis::new("test".to_string());
        AgentManager::parse_output(text_output, &mut analysis);

        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].severity, Severity::Critical);
        assert_eq!(analysis.findings[0].category, Category::Security);
    }

    #[tokio::test]
    async fn test_parse_output_fallback() {
        let plain_output = "Some analysis text without keywords";

        let mut analysis = ReviewAnalysis::new("test".to_string());
        AgentManager::parse_output(plain_output, &mut analysis);

        // Should create a fallback Info finding
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].severity, Severity::Info);
        assert!(analysis.raw_output.is_some());
    }

    #[tokio::test]
    async fn test_run_claude_success() {
        let mock_output = success_output("Warning: Code quality issue\nConsider refactoring");
        let mock_runner = Arc::new(TestCommandRunner::new(mock_output));

        let mut analysis = ReviewAnalysis::new("claude".to_string());
        let result =
            AgentManager::run_claude(123, Path::new("/tmp"), &mut analysis, mock_runner.clone())
                .await;

        assert!(result.is_ok());
        assert!(!analysis.findings.is_empty());

        let calls = mock_runner.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "claude");
        assert!(calls[0].1.contains(&"--model".to_string()));
        assert!(calls[0].1.contains(&"sonnet".to_string()));
    }

    #[tokio::test]
    async fn test_run_claude_error() {
        let mock_output = error_output("Authentication failed");
        let mock_runner = Arc::new(TestCommandRunner::new(mock_output));

        let mut analysis = ReviewAnalysis::new("claude".to_string());
        let result =
            AgentManager::run_claude(123, Path::new("/tmp"), &mut analysis, mock_runner).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChabaError::AgentExecutionError { agent, stderr, .. } => {
                assert_eq!(agent, "claude");
                assert!(stderr.contains("Authentication failed"));
            }
            _ => panic!("Expected AgentExecutionError"),
        }
    }

    #[tokio::test]
    async fn test_agent_manager_new() {
        let config = AgentsConfig::default();
        let manager = AgentManager::new(config);

        // Should have LiveCommandRunner by default
        assert!(Arc::strong_count(&manager.runner) >= 1);
    }

    #[tokio::test]
    async fn test_agent_manager_new_with_runner() {
        let config = AgentsConfig::default();
        let mock_runner: Arc<dyn CommandRunner + Send + Sync> =
            Arc::new(TestCommandRunner::new(success_output("")));

        let manager = AgentManager::new_with_runner(config, mock_runner.clone());

        // Verify runner was injected (Arc count should be 2: manager + test)
        assert_eq!(Arc::strong_count(&manager.runner), 2);
    }
}
