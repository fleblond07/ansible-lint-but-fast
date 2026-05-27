use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

const DEFAULT_MAX_LEN: usize = 160;

/// Lines must not exceed the configured maximum length.
/// Rule ID: yaml[line-length]
pub struct YamlLineLengthRule;

impl Rule for YamlLineLengthRule {
    fn id(&self) -> &str { "yaml[line-length]" }
    fn description(&self) -> &str { "Lines should not exceed the maximum line length" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        file.content
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let len = line.chars().count();
                if len > DEFAULT_MAX_LEN {
                    Some(MatchResult::new(
                        self.id(),
                        format!("Line too long ({len} > {DEFAULT_MAX_LEN} characters)"),
                        file.path.clone(),
                        Location { line: i + 1, column: DEFAULT_MAX_LEN + 1 },
                        self.severity(),
                    ))
                } else {
                    None
                }
            })
            .collect()
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
        YamlLineLengthRule.check_raw_file(&file)
    }

    #[test]
    fn short_line_ok() {
        assert!(lint("name: short\n").is_empty());
    }

    #[test]
    fn long_line_flagged() {
        // "name: " (6) + 200 "x"s = 206 chars > 160 limit
        let long = format!("name: {}\n", "x".repeat(200));
        let results = lint(&long);
        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("206"), "message was: {}", results[0].message);
    }
}
