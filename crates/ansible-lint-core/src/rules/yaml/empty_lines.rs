use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

const MAX_CONSECUTIVE_EMPTY: usize = 2;

/// Too many consecutive blank lines.
/// Rule ID: yaml[empty-lines]
pub struct YamlEmptyLinesRule;

impl Rule for YamlEmptyLinesRule {
    fn id(&self) -> &str { "yaml[empty-lines]" }
    fn description(&self) -> &str { "Too many blank lines" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();
        let mut consecutive = 0usize;

        for (i, line) in file.content.lines().enumerate() {
            if line.trim().is_empty() {
                consecutive += 1;
                if consecutive > MAX_CONSECUTIVE_EMPTY {
                    results.push(MatchResult::new(
                        self.id(),
                        format!("Too many blank lines ({consecutive} > {MAX_CONSECUTIVE_EMPTY})"),
                        file.path.clone(),
                        Location { line: i + 1, column: 1 },
                        self.severity(),
                    ));
                }
            } else {
                consecutive = 0;
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
        let file = LintFile { path: PathBuf::from("t.yml"), content: content.to_string(), kind: FileKind::Tasks };
        YamlEmptyLinesRule.check_raw_file(&file)
    }

    #[test]
    fn two_blank_lines_ok() { assert!(lint("a\n\n\nb\n").is_empty()); }

    #[test]
    fn three_blank_lines_flagged() {
        let r = lint("a\n\n\n\nb\n");
        assert_eq!(r.len(), 1);
    }
}
