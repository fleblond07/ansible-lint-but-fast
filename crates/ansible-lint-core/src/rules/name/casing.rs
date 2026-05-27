use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Task names should start with an uppercase letter.
/// Rule ID: name[casing]
pub struct NameCasingRule;

impl Rule for NameCasingRule {
    fn id(&self) -> &str { "name[casing]" }
    fn description(&self) -> &str { "Task names should start with an uppercase letter" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/name/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if let Some(name) = &task.name {
            if let Some(first_char) = name.chars().next() {
                // Skip names starting with a template variable.
                if first_char == '{' {
                    return vec![];
                }
                if first_char.is_alphabetic() && first_char.is_lowercase() {
                    return vec![MatchResult::new(
                        self.id(),
                        format!("Task name '{name}' should start with an uppercase letter"),
                        file.path.clone(),
                        task.location.clone(),
                        self.severity(),
                    )];
                }
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
        NameCasingRule.check_task(&task, &file)
    }

    #[test]
    fn uppercase_ok() {
        assert!(lint_task("- name: Install nginx\n  apt:\n    name: nginx\n").is_empty());
    }

    #[test]
    fn lowercase_flagged() {
        let r = lint_task("- name: install nginx\n  apt:\n    name: nginx\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn template_start_ok() {
        assert!(lint_task("- name: \"{{ role_name }} - task\"\n  debug:\n    msg: hi\n").is_empty());
    }
}
