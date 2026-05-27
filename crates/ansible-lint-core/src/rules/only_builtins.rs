use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Only ansible.builtin and ansible.posix modules should be used (for portable roles).
/// Rule ID: only-builtins
pub struct OnlyBuiltinsRule;

const ALLOWED_NAMESPACES: &[&str] = &[
    "ansible.builtin.",
    "ansible.posix.",
    "ansible.utils.",
    "ansible.netcommon.",
    "ansible.windows.",
];

// Short names of built-in modules (no namespace prefix).
const BUILTIN_SHORT_NAMES: &[&str] = &[
    "include_tasks", "import_tasks", "include_role", "import_role",
    "import_playbook", "include", "meta", "block",
];

impl Rule for OnlyBuiltinsRule {
    fn id(&self) -> &str { "only-builtins" }
    fn description(&self) -> &str { "Use only ansible.builtin modules for portable roles" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/only-builtins/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["portability"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Shared] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };

        // Short builtins (no namespace) are allowed.
        if BUILTIN_SHORT_NAMES.contains(&module) {
            return vec![];
        }

        // Any FQCN from allowed namespaces is fine.
        if ALLOWED_NAMESPACES.iter().any(|ns| module.starts_with(ns)) {
            return vec![];
        }

        // Short module names without a dot are all the old ansible.builtin.*
        // (they were already checked in fqcn rule; here we flag non-builtin FQCNs).
        if module.contains('.') {
            vec![MatchResult::new(
                self.id(),
                format!("Module '{module}' is not from an allowed namespace; use ansible.builtin.* for portability"),
                file.path.clone(),
                task.location.clone(),
                self.severity(),
            )]
        } else {
            vec![]  // Short name — defer to fqcn rule.
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
        OnlyBuiltinsRule.check_task(&task, &file)
    }

    #[test]
    fn builtin_ok() {
        assert!(lint_task("- name: Debug\n  ansible.builtin.debug:\n    msg: hi\n").is_empty());
    }

    #[test]
    fn community_module_flagged() {
        let r = lint_task("- name: Task\n  community.general.docker_container:\n    name: app\n    state: started\n");
        assert_eq!(r.len(), 1);
    }
}
