use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Task names should not use Jinja2 templating exclusively (the whole name should not be a variable).
/// Rule ID: name[template]
pub struct NameTemplateRule;

impl Rule for NameTemplateRule {
    fn id(&self) -> &str { "name[template]" }
    fn description(&self) -> &str { "Task names should not be a bare Jinja2 template" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/name/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if let Some(name) = &task.name {
            let trimmed = name.trim();
            // A bare template: the entire name is `{{ ... }}`.
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                return vec![MatchResult::new(
                    self.id(),
                    format!("Task name '{name}' is a bare Jinja2 template; add descriptive text"),
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
        NameTemplateRule.check_task(&task, &file)
    }

    #[test]
    fn mixed_template_ok() {
        assert!(lint_task("- name: \"Install {{ pkg }}\"\n  apt:\n    name: \"{{ pkg }}\"\n").is_empty());
    }

    #[test]
    fn bare_template_flagged() {
        let r = lint_task("- name: \"{{ task_name }}\"\n  debug:\n    msg: hi\n");
        assert_eq!(r.len(), 1);
    }
}
