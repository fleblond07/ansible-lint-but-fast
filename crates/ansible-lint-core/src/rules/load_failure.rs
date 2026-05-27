use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Reports files that fail to load/parse as YAML.
/// Rule ID: load-failure[yaml]
pub struct LoadFailureRule;

impl Rule for LoadFailureRule {
    fn id(&self) -> &str { "load-failure[yaml]" }
    fn description(&self) -> &str { "Failed to load or parse YAML file" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/load-failure/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["syntax"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Min] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        // Try to parse YAML; report any parse errors.
        match crate::parser::yaml::parse_yaml_with_positions(&file.content, &file.path.to_string_lossy()) {
            Ok(_) => vec![],
            Err(e) => vec![MatchResult::new(
                self.id(),
                format!("Failed to parse YAML: {e}"),
                file.path.clone(),
                Location { line: 1, column: 1 },
                self.severity(),
            )],
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
        LoadFailureRule.check_raw_file(&file)
    }

    #[test]
    fn valid_yaml_ok() { assert!(lint("---\nfoo: bar\n").is_empty()); }

    #[test]
    fn invalid_yaml_flagged() {
        let r = lint("foo: [unclosed bracket\n");
        assert_eq!(r.len(), 1);
        assert!(r[0].message.contains("parse"));
    }
}
