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
        for result in results {
            match result {
                Ok(Ok(analysis)) => analyses.push(analysis),
                Ok(Err(e)) => eprintln!("Agent execution error: {}", e),
                Err(e) => eprintln!("Task join error: {}", e),
            }
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

        for agent in agents {
            match Self::run_single_agent(agent, pr_number, worktree_path, self.config.timeout).await {
                Ok(analysis) => analyses.push(analysis),
                Err(e) => eprintln!("Agent {} error: {}", agent, e),
            }
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
    /// This is a basic implementation - can be enhanced with JSON parsing
    fn parse_output(output: &str, analysis: &mut ReviewAnalysis) {
        // Store raw output as fallback
        analysis.set_raw_output(output.to_string());

        // Basic pattern matching for common review patterns
        // TODO: Enhance with JSON parsing for structured output

        let lines: Vec<&str> = output.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // Look for severity indicators
            if line.contains("CRITICAL") || line.contains("重大") {
                let title = line.trim().to_string();
                let description = lines.get(i + 1).unwrap_or(&"").trim().to_string();

                let finding = Finding::new(
                    Severity::Critical,
                    Category::Security,
                    title,
                    description,
                );
                analysis.add_finding(finding);
            } else if line.contains("WARNING") || line.contains("警告") {
                let title = line.trim().to_string();
                let description = lines.get(i + 1).unwrap_or(&"").trim().to_string();

                let finding = Finding::new(
                    Severity::Medium,
                    Category::BestPractice,
                    title,
                    description,
                );
                analysis.add_finding(finding);
            }
        }

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
}
