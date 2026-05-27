use crate::parser::task::Task;
use crate::parser::playbook::Play;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Using `become_user` without `become: true` is a configuration error.
/// Rule ID: partial-become
pub struct PartialBecomeRule;

impl Rule for PartialBecomeRule {
    fn id(&self) -> &str { "partial-become" }
    fn description(&self) -> &str { "become_user requires become: true" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/partial-become/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["privilege-escalation", "safety"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if task.become_user.is_some() && task.r#become != Some(true) {
            vec![MatchResult::new(
                self.id(),
                "Task uses 'become_user' without 'become: true'",
                file.path.clone(),
                task.location.clone(),
                self.severity(),
            )]
        } else {
            vec![]
        }
    }

    fn check_play(&self, play: &Play, file: &LintFile) -> Vec<MatchResult> {
        if play.become_user.is_some() && play.r#become != Some(true) {
            vec![MatchResult::new(
                self.id(),
                "Play uses 'become_user' without 'become: true'",
                file.path.clone(),
                play.location.clone(),
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
        PartialBecomeRule.check_task(&task, &file)
    }

    #[test]
    fn become_user_without_become_flagged() {
        let r = lint_task("- name: Task\n  command: whoami\n  become_user: root\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn become_with_become_user_ok() {
        let r = lint_task("- name: Task\n  command: whoami\n  become: true\n  become_user: root\n");
        assert!(r.is_empty());
    }
}
