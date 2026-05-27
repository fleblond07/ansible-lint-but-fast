use serde::Serialize;
use serde_json;

use crate::formatter::Formatter;
use crate::rule::{MatchResult, Severity};

pub struct CodeClimateFormatter;

#[derive(Serialize)]
struct CodeClimateIssue<'a> {
    #[serde(rename = "type")]
    issue_type: &'static str,
    check_name: &'a str,
    description: &'a str,
    categories: Vec<&'static str>,
    severity: &'static str,
    location: CodeClimateLocation<'a>,
    fingerprint: String,
}

#[derive(Serialize)]
struct CodeClimateLocation<'a> {
    path: &'a str,
    lines: CodeClimateLines,
}

#[derive(Serialize)]
struct CodeClimateLines {
    begin: usize,
}

impl Formatter for CodeClimateFormatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        let issues: Vec<CodeClimateIssue> = results.iter().map(|m| {
            let severity = match m.severity {
                Severity::Error => "critical",
                Severity::Warning => "major",
                Severity::Info => "info",
            };
            let fingerprint = format!("{}-{}-{}", m.filename.display(), m.rule_id, m.location.line);
            CodeClimateIssue {
                issue_type: "issue",
                check_name: &m.rule_id,
                description: &m.message,
                categories: vec!["Style"],
                severity,
                location: CodeClimateLocation {
                    path: m.filename.to_str().unwrap_or(""),
                    lines: CodeClimateLines { begin: m.location.line },
                },
                fingerprint,
            }
        }).collect();

        serde_json::to_string_pretty(&issues).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Location, Severity};
    use std::path::PathBuf;

    #[test]
    fn codeclimate_output_valid() {
        let r = MatchResult::new(
            "yaml[truthy]", "Use true/false",
            PathBuf::from("test.yml"),
            Location { line: 3, column: 1 },
            Severity::Warning,
        );
        let fmt = CodeClimateFormatter;
        let out = fmt.format(&[r], false);
        let v: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(v[0]["type"], "issue");
        assert_eq!(v[0]["severity"], "major");
    }
}
