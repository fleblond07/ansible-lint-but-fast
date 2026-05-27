use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

const NON_TRUTHY_BOOLEANS: &[&str] = &[
    "yes", "no", "on", "off",
    "Yes", "No", "On", "Off",
    "YES", "NO", "ON", "OFF",
];

/// Detects YAML boolean values that are not `true` or `false`.
/// Rule ID: yaml[truthy]
pub struct YamlTruthyRule;

impl Rule for YamlTruthyRule {
    fn id(&self) -> &str { "yaml[truthy]" }
    fn description(&self) -> &str { "Truthy value should be true or false" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (line_idx, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments and document markers.
            if trimmed.starts_with('#') || trimmed == "---" || trimmed == "..." {
                continue;
            }

            // Look for `: value` patterns where value is a non-canonical boolean.
            if let Some(colon_pos) = trimmed.find(':') {
                let after_colon = trimmed[colon_pos + 1..].trim();
                // Check if the value (possibly with trailing comment) matches.
                let value_word = after_colon.split_whitespace().next().unwrap_or("").trim_end_matches('#').trim();
                if NON_TRUTHY_BOOLEANS.contains(&value_word) {
                    let col = line.len() - line.trim_start().len() + colon_pos + 2;
                    results.push(MatchResult::new(
                        self.id(),
                        format!("Truthy value should be true or false, found '{value_word}'"),
                        file.path.clone(),
                        Location { line: line_idx + 1, column: col + 1 },
                        self.severity(),
                    ));
                }
            }

            // Also catch list items with bare boolean values: `- yes`
            if let Some(rest) = trimmed.strip_prefix("- ") {
                let val = rest.trim();
                let val_word = val.split_whitespace().next().unwrap_or("").trim_end_matches('#').trim();
                if NON_TRUTHY_BOOLEANS.contains(&val_word) {
                    let col = line.find("- ").unwrap_or(0) + 2;
                    results.push(MatchResult::new(
                        self.id(),
                        format!("Truthy value should be true or false, found '{val_word}'"),
                        file.path.clone(),
                        Location { line: line_idx + 1, column: col + 1 },
                        self.severity(),
                    ));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use std::path::PathBuf;

    fn lint(content: &str) -> Vec<MatchResult> {
        let file = LintFile {
            path: PathBuf::from("test.yml"),
            content: content.to_string(),
            kind: FileKind::Tasks,
        };
        YamlTruthyRule.check_raw_file(&file)
    }

    #[test]
    fn detects_yes() {
        let results = lint("enabled: yes\n");
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("yes"));
    }

    #[test]
    fn detects_no() {
        let results = lint("enabled: no\n");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn allows_true() {
        assert!(lint("enabled: true\n").is_empty());
    }

    #[test]
    fn allows_false() {
        assert!(lint("enabled: false\n").is_empty());
    }

    #[test]
    fn detects_on_off() {
        assert_eq!(lint("enabled: on\n").len(), 1);
        assert_eq!(lint("enabled: off\n").len(), 1);
    }

    #[test]
    fn ignores_comment_lines() {
        assert!(lint("# enabled: yes\n").is_empty());
    }
}
