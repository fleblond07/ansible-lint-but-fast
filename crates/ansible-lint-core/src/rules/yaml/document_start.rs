use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// YAML files must start with the document start marker `---`.
/// Rule ID: yaml[document-start]
pub struct YamlDocumentStartRule;

impl Rule for YamlDocumentStartRule {
    fn id(&self) -> &str { "yaml[document-start]" }
    fn description(&self) -> &str { "Missing document start marker '---'" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let first_non_empty = file.content
            .lines()
            .find(|l| !l.trim().is_empty());

        if first_non_empty.is_none_or(|l| !l.trim_start().starts_with("---")) {
            vec![MatchResult::new(
                self.id(),
                "YAML file should start with '---'",
                file.path.clone(),
                Location { line: 1, column: 1 },
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
        YamlDocumentStartRule.check_raw_file(&file)
    }

    #[test]
    fn with_marker_ok() { assert!(lint("---\n- name: task\n  debug:\n    msg: hi\n").is_empty()); }

    #[test]
    fn without_marker_flagged() {
        let r = lint("- name: task\n  debug:\n    msg: hi\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn empty_file_flagged() {
        assert_eq!(lint("").len(), 1);
    }
}
