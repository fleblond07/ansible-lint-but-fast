use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const COMMAND_MODULES: &[&str] = &[
    "command", "shell", "raw", "script",
    "ansible.builtin.command", "ansible.builtin.shell",
    "ansible.builtin.raw", "ansible.builtin.script",
];

/// `command` and `shell` tasks must have `changed_when` set.
/// Rule ID: no-changed-when
pub struct NoChangedWhenRule;

impl Rule for NoChangedWhenRule {
    fn id(&self) -> &str { "no-changed-when" }
    fn description(&self) -> &str { "Commands should not change things if nothing needs changing" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-changed-when/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["command-shell", "idempotency"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };

        if !COMMAND_MODULES.contains(&module) {
            return vec![];
        }

        // OK if changed_when is set (either as string expression or boolean).
        if task.changed_when.is_some() || task.changed_when_bool.is_some() {
            return vec![];
        }

        // Also OK if `creates` or `removes` is in module args (implies idempotency).
        if let Some(crate::parser::task::ModuleArgs::Mapping(ref args)) = task.module_args {
            if args.contains_key("creates") || args.contains_key("removes") {
                return vec![];
            }
        }

        vec![MatchResult::new(
            self.id(),
            format!("Command task '{module}' should have 'changed_when' to ensure idempotency"),
            file.path.clone(),
            task.location.clone(),
            self.severity(),
        )]
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
        NoChangedWhenRule.check_task(&task, &file)
    }

    #[test]
    fn command_without_changed_when_flagged() {
        let r = lint_task("- name: Run cmd\n  command: echo hello\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn command_with_changed_when_ok() {
        let r = lint_task("- name: Run cmd\n  command: echo hello\n  changed_when: false\n");
        assert!(r.is_empty());
    }

    #[test]
    fn debug_not_affected() {
        assert!(lint_task("- name: Debug\n  debug:\n    msg: hi\n").is_empty());
    }
}
