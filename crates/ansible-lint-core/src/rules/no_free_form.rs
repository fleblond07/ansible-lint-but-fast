use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

// Modules that legitimately accept free-form arguments.
const FREE_FORM_OK: &[&str] = &[
    "command", "shell", "raw", "script", "set_fact",
    "ansible.builtin.command", "ansible.builtin.shell",
    "ansible.builtin.raw", "ansible.builtin.script",
    "ansible.builtin.set_fact",
    // These use key=value inline style which is also allowed.
    "include_vars", "ansible.builtin.include_vars",
];

/// Modules that do not support free-form args should use YAML mapping style.
/// Rule ID: no-free-form
pub struct NoFreeFormRule;

impl Rule for NoFreeFormRule {
    fn id(&self) -> &str { "no-free-form" }
    fn description(&self) -> &str { "Avoid using free-form module arguments; use key/value mapping" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-free-form/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["syntax"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };

        if FREE_FORM_OK.contains(&module) {
            return vec![];
        }

        if let Some(ModuleArgs::FreeForm(args)) = &task.module_args {
            // A free-form string that contains `=` is the old key=value style.
            if args.contains('=') || !args.trim().is_empty() {
                return vec![MatchResult::new(
                    self.id(),
                    format!("Module '{module}' should not use free-form args; use YAML mapping"),
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
        NoFreeFormRule.check_task(&task, &file)
    }

    #[test]
    fn mapping_style_ok() {
        assert!(lint_task("- name: Install\n  apt:\n    name: nginx\n    state: present\n").is_empty());
    }

    #[test]
    fn command_free_form_ok() {
        assert!(lint_task("- name: Run\n  command: echo hello\n").is_empty());
    }

    #[test]
    fn apt_free_form_flagged() {
        let r = lint_task("- name: Install\n  apt: name=nginx state=present\n");
        assert_eq!(r.len(), 1);
    }
}
