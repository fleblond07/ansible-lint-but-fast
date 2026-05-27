use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const PATH_MODULES: &[&str] = &[
    "copy", "template", "script", "include_vars",
    "ansible.builtin.copy", "ansible.builtin.template",
    "ansible.builtin.script", "ansible.builtin.include_vars",
];

/// Tasks referencing files should use absolute paths or role-relative paths, not `../`.
/// Rule ID: no-relative-paths
pub struct NoRelativePathsRule;

impl Rule for NoRelativePathsRule {
    fn id(&self) -> &str { "no-relative-paths" }
    fn description(&self) -> &str { "Avoid relative paths in roles; use role-specific vars or absolute paths" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-relative-paths/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if !PATH_MODULES.contains(&module) {
            return vec![];
        }

        let contains_relative = match &task.module_args {
            Some(ModuleArgs::Mapping(args)) => {
                args.values().any(|v| {
                    v.as_str().is_some_and(|s| s.contains("../") || s.starts_with("./"))
                })
            }
            Some(ModuleArgs::FreeForm(s)) => s.contains("../") || s.starts_with("./"),
            None => false,
        };

        if contains_relative {
            vec![MatchResult::new(
                self.id(),
                format!("'{module}' task uses a relative path; use absolute path or role-relative variables"),
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
        NoRelativePathsRule.check_task(&task, &file)
    }

    #[test]
    fn absolute_path_ok() {
        assert!(lint_task("- name: Copy\n  copy:\n    src: /etc/file\n    dest: /tmp/file\n").is_empty());
    }

    #[test]
    fn relative_path_flagged() {
        let r = lint_task("- name: Copy\n  copy:\n    src: ../files/foo\n    dest: /tmp/foo\n");
        assert_eq!(r.len(), 1);
    }
}
