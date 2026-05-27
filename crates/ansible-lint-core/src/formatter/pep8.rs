use crate::formatter::Formatter;
use crate::rule::MatchResult;

/// PEP8 / flake8-style output: `path:line:col: CODE message`
pub struct Pep8Formatter;

impl Formatter for Pep8Formatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        results.iter()
            .map(|m| {
                // Convert rule ID to a short code: e.g. "yaml[truthy]" → "YAML001"
                let code = rule_to_code(&m.rule_id);
                format!(
                    "{}:{}:{}: {} {}",
                    m.filename.display(),
                    m.location.line,
                    m.location.column,
                    code,
                    m.message,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn rule_to_code(rule_id: &str) -> String {
    // Generate a stable alphabetic code from the rule ID.
    // ansible-lint uses the rule ID directly in pep8 output.
    rule_id.to_string()
}

/// Markdown report formatter.
pub struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        if results.is_empty() {
            return "# ansible-lint report\n\nNo violations found.\n".to_string();
        }

        let mut out = String::from("# ansible-lint report\n\n");
        out.push_str("| File | Line | Rule | Severity | Message |\n");
        out.push_str("|------|------|------|----------|---------|\n");

        for m in results {
            out.push_str(&format!(
                "| {} | {} | `{}` | {} | {} |\n",
                m.filename.display(),
                m.location.line,
                m.rule_id,
                m.severity,
                m.message.replace('|', "\\|"),
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Location, Severity};
    use std::path::PathBuf;

    fn make_result() -> MatchResult {
        MatchResult::new(
            "yaml[truthy]", "Use true/false",
            PathBuf::from("test.yml"),
            Location { line: 3, column: 1 },
            Severity::Warning,
        )
    }

    #[test]
    fn pep8_format() {
        let out = Pep8Formatter.format(&[make_result()], false);
        assert!(out.contains("test.yml:3:1:"));
        assert!(out.contains("yaml[truthy]"));
    }

    #[test]
    fn markdown_format() {
        let out = MarkdownFormatter.format(&[make_result()], false);
        assert!(out.contains("# ansible-lint report"));
        assert!(out.contains("yaml[truthy]"));
    }

    #[test]
    fn markdown_empty() {
        let out = MarkdownFormatter.format(&[], false);
        assert!(out.contains("No violations found"));
    }
}
