use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const PACKAGE_MODULES: &[&str] = &[
    "apt", "yum", "dnf", "package", "pip", "gem",
    "ansible.builtin.apt", "ansible.builtin.yum",
    "ansible.builtin.dnf", "ansible.builtin.package",
    "ansible.builtin.pip",
];

/// Package install tasks should not use `state: latest`.
/// Rule ID: package-latest
pub struct PackageLatestRule;

impl Rule for PackageLatestRule {
    fn id(&self) -> &str { "package-latest" }
    fn description(&self) -> &str { "Package installs should not use state: latest" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/package-latest/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idempotency"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if !PACKAGE_MODULES.contains(&module) {
            return vec![];
        }

        let uses_latest = match &task.module_args {
            Some(ModuleArgs::Mapping(args)) => {
                args.get("state").and_then(|v| v.as_str()) == Some("latest")
            }
            Some(ModuleArgs::FreeForm(s)) => s.contains("state=latest"),
            None => false,
        };

        if uses_latest {
            vec![MatchResult::new(
                self.id(),
                format!("'{module}' should not use state: latest (breaks idempotency)"),
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
        PackageLatestRule.check_task(&task, &file)
    }

    #[test]
    fn state_latest_flagged() {
        let r = lint_task("- name: Install\n  apt:\n    name: nginx\n    state: latest\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn state_present_ok() {
        let r = lint_task("- name: Install\n  apt:\n    name: nginx\n    state: present\n");
        assert!(r.is_empty());
    }
}
