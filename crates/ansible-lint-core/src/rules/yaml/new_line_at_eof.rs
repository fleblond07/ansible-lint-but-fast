use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// YAML files must end with a newline character.
/// Rule ID: yaml[new-line-at-end-of-file]
pub struct YamlNewLineAtEofRule;

impl Rule for YamlNewLineAtEofRule {
    fn id(&self) -> &str { "yaml[new-line-at-end-of-file]" }
    fn description(&self) -> &str { "File must end with a newline" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        if file.content.is_empty() {
            return vec![];
        }
        if !file.content.ends_with('\n') {
            let line_count = file.content.lines().count();
            vec![MatchResult::new(
                self.id(),
                "File does not end with a newline",
                file.path.clone(),
                Location { line: line_count, column: file.content.lines().last().map_or(1, |l| l.len() + 1) },
                self.severity(),
            )]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use std::path::PathBuf;

    fn lint(content: &str) -> Vec<MatchResult> {
        let file = LintFile { path: PathBuf::from("t.yml"), content: content.to_string(), kind: FileKind::Tasks };
        YamlNewLineAtEofRule.check_raw_file(&file)
    }

    #[test]
    fn ends_with_newline_ok() { assert!(lint("---\nfoo: bar\n").is_empty()); }

    #[test]
    fn no_trailing_newline_flagged() {
        assert_eq!(lint("---\nfoo: bar").len(), 1);
    }

    #[test]
    fn empty_file_ok() { assert!(lint("").is_empty()); }
}
