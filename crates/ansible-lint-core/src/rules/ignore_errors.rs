use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Avoid using `ignore_errors: true`; use `failed_when` instead.
/// Rule ID: ignore-errors
pub struct IgnoreErrorsRule;

impl Rule for IgnoreErrorsRule {
    fn id(&self) -> &str { "ignore-errors" }
    fn description(&self) -> &str { "Avoid ignore_errors; use failed_when instead" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/ignore-errors/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if let Some(val) = task.raw.get("ignore_errors") {
            if val.as_bool() == Some(true) {
                return vec![MatchResult::new(
                    self.id(),
                    "Avoid 'ignore_errors: true'; use 'failed_when' with explicit conditions instead",
                    file.path.clone(),
                    task.location.clone(),
                    self.severity(),
                )];
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use crate::parser::yaml::parse_yaml_with_positions;
    use crate::parser::task::parse_task;
    use std::path::PathBuf;

    fn lint_task(yaml: &str) -> Vec<MatchResult> {
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        let file = LintFile { path: PathBuf::from("test.yml"), content: yaml.to_string(), kind: FileKind::Tasks };
        IgnoreErrorsRule.check_task(&task, &file)
    }

    #[test]
    fn ignore_errors_true_flagged() {
        let r = lint_task("- name: Task\n  command: echo hi\n  ignore_errors: true\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn no_ignore_errors_ok() {
        assert!(lint_task("- name: Task\n  command: echo hi\n").is_empty());
    }
}
