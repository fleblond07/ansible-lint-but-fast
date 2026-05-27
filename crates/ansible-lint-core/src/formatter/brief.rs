use owo_colors::OwoColorize;

use crate::formatter::Formatter;
use crate::rule::{MatchResult, Severity};

/// `path:line:col: severity  [rule-id] message`
pub struct BriefFormatter;

impl Formatter for BriefFormatter {
    fn format(&self, results: &[MatchResult], color: bool) -> String {
        results.iter().map(|m| format_line(m, color)).collect::<Vec<_>>().join("\n")
    }
}

/// Only prints file:line counts — no per-match lines.
pub struct QuietFormatter;

impl Formatter for QuietFormatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        use std::collections::HashMap;
        let mut counts: HashMap<&std::path::Path, usize> = HashMap::new();
        for m in results {
            *counts.entry(m.filename.as_path()).or_insert(0) += 1;
        }
        let mut lines: Vec<String> = counts
            .into_iter()
            .map(|(p, n)| format!("{}: {n} violation(s)", p.display()))
            .collect();
        lines.sort();
        lines.join("\n")
    }
}

pub fn format_line(m: &MatchResult, color: bool) -> String {
    let loc = format!("{}:{}:{}", m.filename.display(), m.location.line, m.location.column);
    let sev = severity_str(&m.severity, color);
    let rule = format!("[{}]", m.rule_id);
    format!("{loc}: {sev}  {rule} {}", m.message)
}

fn severity_str(sev: &Severity, color: bool) -> String {
    if !color {
        return sev.to_string();
    }
    match sev {
        Severity::Error => "error".red().bold().to_string(),
        Severity::Warning => "warning".yellow().to_string(),
        Severity::Info => "info".cyan().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Location;
    use std::path::PathBuf;

    fn make_result(rule: &str, msg: &str, line: usize) -> MatchResult {
        MatchResult::new(rule, msg, PathBuf::from("test.yml"), Location { line, column: 1 }, Severity::Error)
    }

    #[test]
    fn test_brief_format() {
        let r = make_result("yaml[truthy]", "Use true/false", 5);
        let f = BriefFormatter;
        let out = f.format(&[r], false);
        assert!(out.contains("test.yml:5:1"));
        assert!(out.contains("[yaml[truthy]]"));
        assert!(out.contains("Use true/false"));
    }
}
