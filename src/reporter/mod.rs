use crate::diff::{ContractChange, DiffResult, Severity};
use colored::*;
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Markdown,
}

impl OutputFormat {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "json" => Self::Json,
            "markdown" | "md" => Self::Markdown,
            _ => Self::Table,
        }
    }
}

pub fn render_report(result: &DiffResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => render_json(result),
        OutputFormat::Markdown => render_markdown(result),
        OutputFormat::Table => render_table(result),
    }
}

fn render_json(result: &DiffResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}

fn render_markdown(result: &DiffResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Contract Regression Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Baseline**: {}", result.baseline_title);
    let _ = writeln!(out, "- **Current**: {}", result.current_title);
    let _ = writeln!(
        out,
        "- **Status**: {}",
        if result.is_breaking {
            "🚨 **BREAKING CONTRACT REGRESSION DETECTED**"
        } else {
            "✅ **BACKWARD COMPATIBLE**"
        }
    );
    let _ = writeln!(out, "- **Risk Level**: `{}` (Score: {}/100)", result.risk_level, result.severity_score);
    let _ = writeln!(
        out,
        "- **Summary**: {} breaking changes, {} safe changes",
        result.breaking_count, result.non_breaking_count
    );
    let _ = writeln!(out);

    if result.changes.is_empty() {
        let _ = writeln!(out, "*No contract changes detected.*");
        return out;
    }

    let _ = writeln!(out, "## Detailed Contract Changes");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Severity | Breaking | Target | Description |");
    let _ = writeln!(out, "| :--- | :---: | :--- | :--- |");

    for c in &result.changes {
        let sev_badge = match c.severity {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
            Severity::Safe => "SAFE",
        };
        let breaking_icon = if c.is_breaking { "🚨 YES" } else { "🛡️ NO" };
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            sev_badge, breaking_icon, c.target, c.message
        );
    }

    out
}

fn render_table(result: &DiffResult) -> String {
    let mut out = String::new();

    let banner = if result.is_breaking {
        "🚨 BREAKING CONTRACT REGRESSION DETECTED".red().bold()
    } else {
        "✅ CONTRACT IS BACKWARD COMPATIBLE".green().bold()
    };

    let _ = writeln!(out, "================================================================================");
    let _ = writeln!(out, "                             CONTRACT-GUARD REPORT                              ");
    let _ = writeln!(out, "================================================================================");
    let _ = writeln!(out, " Status:       {banner}");
    let _ = writeln!(out, " Baseline:     {}", result.baseline_title.cyan());
    let _ = writeln!(out, " Current:      {}", result.current_title.cyan());
    let _ = writeln!(
        out,
        " Risk Level:   {} (Score: {}/100)",
        color_risk(&result.risk_level),
        result.severity_score
    );
    let _ = writeln!(
        out,
        " Impact:       {} breaking, {} backward-compatible mutations",
        result.breaking_count.to_string().red().bold(),
        result.non_breaking_count.to_string().green()
    );
    let _ = writeln!(out, "--------------------------------------------------------------------------------");

    if result.changes.is_empty() {
        let _ = writeln!(out, " No changes found. API contracts match exactly.");
        let _ = writeln!(out, "================================================================================");
        return out;
    }

    let _ = writeln!(
        out,
        "{:<10} {:<10} {:<32} {}",
        "SEVERITY".bold(),
        "BREAKING".bold(),
        "TARGET".bold(),
        "CHANGE DESCRIPTION".bold()
    );
    let _ = writeln!(out, "--------------------------------------------------------------------------------");

    for c in &result.changes {
        let (sev_str, breaking_str) = format_change_badges(c);
        let target_display = truncate_target(&c.target, 30);
        let _ = writeln!(
            out,
            "{:<10} {:<10} {:<32} {}",
            sev_str, breaking_str, target_display, c.message
        );
    }

    let _ = writeln!(out, "================================================================================");
    out
}

fn color_risk(risk: &str) -> ColoredString {
    match risk {
        "CRITICAL" => risk.red().bold(),
        "HIGH" => risk.bright_red().bold(),
        "MEDIUM" => risk.yellow().bold(),
        "LOW" => risk.cyan(),
        _ => risk.green().bold(),
    }
}

fn format_change_badges(c: &ContractChange) -> (ColoredString, ColoredString) {
    let sev_str = match c.severity {
        Severity::Critical => "CRITICAL".red().bold(),
        Severity::High => "HIGH".bright_red().bold(),
        Severity::Medium => "MEDIUM".yellow(),
        Severity::Low => "LOW".cyan(),
        Severity::Safe => "SAFE".green(),
    };

    let breaking_str = if c.is_breaking {
        "YES".red().bold()
    } else {
        "NO".green()
    };

    (sev_str, breaking_str)
}

fn truncate_target(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
