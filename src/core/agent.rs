use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

use crate::config::AgentsConfig;
use crate::core::review_analysis::{ReviewAnalysis, Finding, Severity, Category};
use crate::error::{ChabaError, Result};

pub struct AgentManager {
    config: AgentsConfig,
}

impl AgentManager {
    /// Create a new AgentManager
    pub fn new(config: AgentsConfig) -> Self {
        AgentManager { config }
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
        let mut tasks = Vec::new();

        for agent in agents {
            let agent = agent.clone();
            let worktree_path = worktree_path.to_path_buf();
            let timeout = self.config.timeout;

            tasks.push(tokio::spawn(async move {
                Self::run_single_agent(&agent, pr_number, &worktree_path, timeout).await
            }));
        }

        let results = futures::future::join_all(tasks).await;

        let mut analyses = Vec::new();
        let mut errors = Vec::new();

        for (idx, result) in results.into_iter().enumerate() {
            let agent_name = &agents[idx];
            match result {
                Ok(Ok(analysis)) => {
                    eprintln!("✓ {} completed analysis", agent_name);
                    analyses.push(analysis);
                }
                Ok(Err(e)) => {
                    eprintln!("✗ {} failed: {}", agent_name, e);
                    errors.push((agent_name.clone(), e.to_string()));
                }
                Err(e) => {
                    eprintln!("✗ {} task failed: {}", agent_name, e);
                    errors.push((agent_name.clone(), e.to_string()));
                }
            }
        }

        if !errors.is_empty() && analyses.is_empty() {
            // All agents failed
            eprintln!("\n⚠️  All agents failed to complete analysis");
            eprintln!("Review the errors above and check:");
            eprintln!("  - Agent CLI tools are installed (claude, codex, gemini)");
            eprintln!("  - Network connectivity");
            eprintln!("  - Agent timeout setting (current: {}s)", self.config.timeout);
        } else if !errors.is_empty() {
            // Some agents failed
            eprintln!("\n⚠️  {} agent(s) failed, {} succeeded", errors.len(), analyses.len());
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
        let mut analyses = Vec::new();
        let mut errors = Vec::new();

        for agent in agents {
            eprintln!("Running {} analysis...", agent);
            match Self::run_single_agent(agent, pr_number, worktree_path, self.config.timeout).await {
                Ok(analysis) => {
                    eprintln!("✓ {} completed", agent);
                    analyses.push(analysis);
                }
                Err(e) => {
                    eprintln!("✗ {} failed: {}", agent, e);
                    errors.push((agent.clone(), e.to_string()));
                }
            }
        }

        if !errors.is_empty() && analyses.is_empty() {
            eprintln!("\n⚠️  All agents failed to complete analysis");
            eprintln!("Check agent CLI tool installations and network connectivity");
        } else if !errors.is_empty() {
            eprintln!("\n⚠️  {} agent(s) failed, {} succeeded", errors.len(), analyses.len());
        }

        Ok(analyses)
    }

    /// Run a single agent with timeout
    async fn run_single_agent(
        agent: &str,
        pr_number: u32,
        worktree_path: &Path,
        timeout_secs: u64,
    ) -> Result<ReviewAnalysis> {
        let timeout = Duration::from_secs(timeout_secs);

        let result = tokio::time::timeout(
            timeout,
            Self::execute_agent(agent, pr_number, worktree_path),
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
    ) -> Result<ReviewAnalysis> {
        let mut analysis = ReviewAnalysis::new(agent.to_string());

        match agent {
            "claude" => Self::run_claude(pr_number, worktree_path, &mut analysis).await?,
            "codex" => Self::run_codex(pr_number, worktree_path, &mut analysis).await?,
            "gemini" => Self::run_gemini(pr_number, worktree_path, &mut analysis).await?,
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
    ) -> Result<()> {
        let prompt = format!(
            "PR #{} のコードレビューを実施してください。品質、セキュリティ、パフォーマンスの観点から分析し、改善点を指摘してください。",
            pr_number
        );

        let output = Command::new("claude")
            .current_dir(worktree_path)
            .args(["--model", "sonnet", "--yes", &prompt])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            analysis.set_raw_output(stderr.to_string());
        }

        Ok(())
    }

    /// Run Codex agent
    async fn run_codex(
        pr_number: u32,
        worktree_path: &Path,
        analysis: &mut ReviewAnalysis,
    ) -> Result<()> {
        let prompt = format!(
            "このPR #{}のコードをレビューしてください。バグ、セキュリティ問題、ベストプラクティス違反を指摘してください。",
            pr_number
        );

        let output = Command::new("codex")
            .current_dir(worktree_path)
            .args([
                "exec",
                "--full-auto",
                "--sandbox",
                "read-only",
                &prompt,
            ])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            analysis.set_raw_output(stderr.to_string());
        }

        Ok(())
    }

    /// Run Gemini agent
    async fn run_gemini(
        pr_number: u32,
        worktree_path: &Path,
        analysis: &mut ReviewAnalysis,
    ) -> Result<()> {
        let prompt = format!(
            "このPR #{}を戦略的視点からレビューしてください。アーキテクチャ、設計パターン、拡張性について分析してください。",
            pr_number
        );

        let output = Command::new("gemini")
            .current_dir(worktree_path)
            .args([
                "-m",
                "gemini-2.0-flash-001",
                "-s",
                "-y",
                "-p",
                &prompt,
            ])
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Self::parse_output(&stdout, analysis);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            analysis.set_raw_output(stderr.to_string());
        }

        Ok(())
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
