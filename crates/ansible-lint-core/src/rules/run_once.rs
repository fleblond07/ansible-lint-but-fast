use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// `run_once: true` with `delegate_to` or in a loop can cause surprising behaviour.
/// Rule ID: run-once[task]
pub struct RunOnceRule;

impl Rule for RunOnceRule {
    fn id(&self) -> &str { "run-once[task]" }
    fn description(&self) -> &str { "run_once with delegate_to may have unexpected results" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/run-once/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom", "safety"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let run_once = task.raw.get("run_once")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !run_once {
            return vec![];
        }

        let has_loop = task.raw.contains_key("loop")
            || task.raw.contains_key("with_items")
            || task.raw.contains_key("with_list");

        if has_loop {
            return vec![MatchResult::new(
                self.id(),
                "Using run_once with a loop may produce unexpected results",
                file.path.clone(),
                task.location.clone(),
                self.severity(),
            )];
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
        RunOnceRule.check_task(&task, &file)
    }

    #[test]
    fn run_once_with_loop_flagged() {
        let r = lint_task("- name: T\n  debug:\n    msg: hi\n  run_once: true\n  loop: [1, 2]\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn run_once_no_loop_ok() {
        assert!(lint_task("- name: T\n  debug:\n    msg: hi\n  run_once: true\n").is_empty());
    }
}
