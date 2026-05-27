use serde::Serialize;
use serde_json;

use crate::formatter::Formatter;
use crate::rule::MatchResult;

pub struct JsonFormatter;

#[derive(Serialize)]
struct JsonMatch<'a> {
    rule: &'a str,
    message: &'a str,
    filename: String,
    line: usize,
    column: usize,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<&'a str>,
}

impl Formatter for JsonFormatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        let items: Vec<JsonMatch> = results.iter().map(|m| JsonMatch {
            rule: &m.rule_id,
            message: &m.message,
            filename: m.filename.display().to_string(),
            line: m.location.line,
            column: m.location.column,
            severity: m.severity.to_string(),
            tag: m.tag.as_deref(),
            task: m.task_name.as_deref(),
        }).collect();

        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Location, Severity};
    use std::path::PathBuf;

    #[test]
    fn test_json_format() {
        let r = MatchResult::new(
            "yaml[truthy]",
            "Use true/false",
            PathBuf::from("test.yml"),
            Location { line: 3, column: 8 },
            Severity::Error,
        );
        let fmt = JsonFormatter;
        let out = fmt.format(&[r], false);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["rule"], "yaml[truthy]");
        assert_eq!(parsed[0]["line"], 3);
    }
}
