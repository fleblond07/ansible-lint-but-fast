use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// All tasks should have a name.
/// Rule ID: name[missing]
pub struct NameMissingRule;

impl Rule for NameMissingRule {
    fn id(&self) -> &str { "name[missing]" }
    fn description(&self) -> &str { "All tasks should have a name" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/name/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if task.name.is_none() {
            vec![MatchResult::new(
                self.id(),
                "All tasks should be named",
                file.path.clone(),
                task.location.clone(),
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
    use crate::parser::yaml::parse_yaml_with_positions;
    use crate::parser::task::parse_task;
    use std::path::PathBuf;

    fn lint_task(yaml: &str) -> Vec<MatchResult> {
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        let file = LintFile { path: PathBuf::from("test.yml"), content: yaml.to_string(), kind: FileKind::Tasks };
        NameMissingRule.check_task(&task, &file)
    }

    #[test]
    fn named_task_ok() {
        let r = lint_task("- name: Do thing\n  debug:\n    msg: hi\n");
        assert!(r.is_empty());
    }

    #[test]
    fn unnamed_task_flagged() {
        let r = lint_task("- debug:\n    msg: hi\n");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].rule_id, "name[missing]");
    }
}
