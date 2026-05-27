use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const FILE_MODULES: &[&str] = &[
    "file", "copy", "template", "get_url", "unarchive",
    "ansible.builtin.file", "ansible.builtin.copy",
    "ansible.builtin.template", "ansible.builtin.get_url",
    "ansible.builtin.unarchive",
];

/// File tasks must set permissions explicitly to avoid world-writable files.
/// Rule ID: risky-file-permissions
pub struct RiskyFilePermissionsRule;

impl Rule for RiskyFilePermissionsRule {
    fn id(&self) -> &str { "risky-file-permissions" }
    fn description(&self) -> &str { "File permissions should be specified explicitly" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/risky-file-permissions/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["safety"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if !FILE_MODULES.contains(&module) {
            return vec![];
        }

        let args = match &task.module_args {
            Some(ModuleArgs::Mapping(m)) => m,
            _ => return vec![],
        };

        // Requires explicit `mode` unless `state: absent` or `state: directory` with defaults.
        let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("file");
        if state == "absent" || state == "link" {
            return vec![];
        }

        if !args.contains_key("mode") {
            return vec![MatchResult::new(
                self.id(),
                format!("'{module}' task does not have explicit permissions (mode)"),
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
        RiskyFilePermissionsRule.check_task(&task, &file)
    }

    #[test]
    fn file_without_mode_flagged() {
        let r = lint_task("- name: Create file\n  file:\n    path: /tmp/foo\n    state: present\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn file_with_mode_ok() {
        let r = lint_task("- name: Create file\n  file:\n    path: /tmp/foo\n    state: present\n    mode: '0644'\n");
        assert!(r.is_empty());
    }

    #[test]
    fn file_absent_ok() {
        let r = lint_task("- name: Remove\n  file:\n    path: /tmp/foo\n    state: absent\n");
        assert!(r.is_empty());
    }
}
