use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Shell commands using pipes should have `pipefail` set.
/// Rule ID: risky-shell-pipe
pub struct RiskyShellPipeRule;

const SHELL_MODULES: &[&str] = &[
    "shell", "ansible.builtin.shell",
];

impl Rule for RiskyShellPipeRule {
    fn id(&self) -> &str { "risky-shell-pipe" }
    fn description(&self) -> &str { "Shells that use pipes without pipefail are risky" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/risky-shell-pipe/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["command-shell", "safety"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if !SHELL_MODULES.contains(&module) {
            return vec![];
        }

        let cmd = match &task.module_args {
            Some(ModuleArgs::FreeForm(s)) => s.clone(),
            Some(ModuleArgs::Mapping(m)) => m.get("cmd")
                .or_else(|| m.get("_raw_params"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            None => return vec![],
        };

        if cmd.contains('|') && !cmd.contains("pipefail") {
            return vec![MatchResult::new(
                self.id(),
                "Shell command uses a pipe without 'set -o pipefail'",
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
        RiskyShellPipeRule.check_task(&task, &file)
    }

    #[test]
    fn pipe_without_pipefail_flagged() {
        let r = lint_task("- name: Pipe\n  shell: cat file | grep foo\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn pipe_with_pipefail_ok() {
        let r = lint_task("- name: Safe pipe\n  shell: set -o pipefail && cat file | grep foo\n");
        assert!(r.is_empty());
    }

    #[test]
    fn no_pipe_ok() {
        assert!(lint_task("- name: No pipe\n  shell: echo hello\n").is_empty());
    }
}
