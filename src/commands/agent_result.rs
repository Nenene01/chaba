use crate::core::review_analysis::{Severity, Category};
use crate::core::state::State;
use crate::error::{ChabaError, Result};

pub async fn execute(pr: u32) -> Result<()> {
    let state = State::load()?;

    let review = state
        .get_review(pr)
        .ok_or_else(|| ChabaError::PrNotFound(pr))?;

    if review.agent_analyses.is_empty() {
        println!("No AI agent analysis found for PR #{}", pr);
        println!("\nTip: Run 'chaba review --pr {} --with-agent' to generate analysis", pr);
        return Ok(());
    }

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  AI Agent Review Results - PR #{}                          ", pr);
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    println!("📊 Review Environment:");
    println!("  Branch: {}", review.branch);
    println!("  Path: {}", review.worktree_path.display());
    if let Some(project_type) = &review.project_type {
        println!("  Project Type: {}", project_type);
    }
    println!();

    for analysis in &review.agent_analyses {
        print_agent_analysis(analysis);
    }

    // Summary statistics
    print_summary(&review.agent_analyses);

    Ok(())
}

fn print_agent_analysis(analysis: &crate::core::review_analysis::ReviewAnalysis) {
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ 🤖 Agent: {:<50} │", analysis.agent);
    println!("│ 🕐 Time: {:<51} │", &analysis.timestamp[..19]);
    if let Some(score) = analysis.score {
        println!("│ ⭐ Score: {:.1}/5.0{:<44} │", score, "");
    }
    println!("└────────────────────────────────────────────────────────────┘");

    if analysis.findings.is_empty() {
        println!("  No structured findings");
        if let Some(raw) = &analysis.raw_output {
            println!("\n  Raw output:");
            for line in raw.lines().take(5) {
                println!("    {}", line);
            }
            if raw.lines().count() > 5 {
                println!("    ... ({} more lines)", raw.lines().count() - 5);
            }
        }
        println!();
        return;
    }

    // Group findings by severity
    let critical = analysis.findings.iter()
        .filter(|f| f.severity == Severity::Critical)
        .collect::<Vec<_>>();
    let high = analysis.findings.iter()
        .filter(|f| f.severity == Severity::High)
        .collect::<Vec<_>>();
    let medium = analysis.findings.iter()
        .filter(|f| f.severity == Severity::Medium)
        .collect::<Vec<_>>();
    let low = analysis.findings.iter()
        .filter(|f| f.severity == Severity::Low)
        .collect::<Vec<_>>();
    let info = analysis.findings.iter()
        .filter(|f| f.severity == Severity::Info)
        .collect::<Vec<_>>();

    if !critical.is_empty() {
        println!("\n  🔴 CRITICAL ({}):", critical.len());
        for finding in critical {
            print_finding(finding);
        }
    }

    if !high.is_empty() {
        println!("\n  🟠 HIGH ({}):", high.len());
        for finding in high {
            print_finding(finding);
        }
    }

    if !medium.is_empty() {
        println!("\n  🟡 MEDIUM ({}):", medium.len());
        for finding in medium {
            print_finding(finding);
        }
    }

    if !low.is_empty() {
        println!("\n  🔵 LOW ({}):", low.len());
        for finding in low {
            print_finding(finding);
        }
    }

    if !info.is_empty() {
        println!("\n  ⚪ INFO ({}):", info.len());
        for finding in info {
            print_finding(finding);
        }
    }

    println!();
}

fn print_finding(finding: &crate::core::review_analysis::Finding) {
    print!("    • {}", finding.title);

    if let Some(file) = &finding.file {
        if let Some(line) = finding.line {
            print!(" ({}:{})", file, line);
        } else {
            print!(" ({})", file);
        }
    }

    println!();

    if !finding.description.is_empty() {
        println!("      {}", finding.description);
    }

    if let Some(suggestion) = &finding.suggestion {
        println!("      💡 Suggestion: {}", suggestion);
    }
}

fn print_summary(analyses: &[crate::core::review_analysis::ReviewAnalysis]) {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Summary                                                        ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let total_findings: usize = analyses.iter().map(|a| a.findings.len()).sum();
    let total_critical: usize = analyses.iter()
        .map(|a| a.count_by_severity(&Severity::Critical))
        .sum();
    let total_high: usize = analyses.iter()
        .map(|a| a.count_by_severity(&Severity::High))
        .sum();

    println!("  Total Agents: {}", analyses.len());
    println!("  Total Findings: {}", total_findings);

    if total_critical > 0 || total_high > 0 {
        println!("\n  ⚠️  Attention Required:");
        if total_critical > 0 {
            println!("    🔴 {} Critical issue(s)", total_critical);
        }
        if total_high > 0 {
            println!("    🟠 {} High priority issue(s)", total_high);
        }
    } else {
        println!("\n  ✅ No critical or high priority issues found");
    }

    // Category breakdown
    let categories = [
        Category::Security,
        Category::Performance,
        Category::BestPractice,
        Category::CodeQuality,
        Category::Architecture,
        Category::Testing,
        Category::Documentation,
    ];

    let mut has_categories = false;
    for category in &categories {
        let count: usize = analyses.iter()
            .map(|a| a.count_by_category(category))
            .sum();
        if count > 0 {
            if !has_categories {
                println!("\n  Categories:");
                has_categories = true;
            }
            println!("    • {:?}: {}", category, count);
        }
    }

    println!();
}
