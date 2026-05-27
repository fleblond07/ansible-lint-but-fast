use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// The `name:` key should be the first key in a task mapping.
/// Rule ID: key-order[task]
pub struct KeyOrderRule;

impl Rule for KeyOrderRule {
    fn id(&self) -> &str { "key-order[task]" }
    fn description(&self) -> &str { "The 'name' key should come first in a task" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/key-order/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        // Check if name is present but not the first key by scanning raw.
        // Since HashMap doesn't preserve order, we look for the name in the raw YAML.
        // We use a simple heuristic: scan the file lines around task.location.line.
        let line_num = task.location.line;
        let lines: Vec<&str> = file.content.lines().collect();

        if task.name.is_none() {
            return vec![]; // no-name tasks are caught by name[missing]
        }

        // Find the first non-empty, non-comment key in the task block.
        let start = line_num.saturating_sub(1);
        let mut first_key: Option<&str> = None;

        for line in &lines[start..] {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            // List item start.
            let check = trimmed.strip_prefix("- ").unwrap_or(trimmed);
            if let Some(colon) = check.find(':') {
                first_key = Some(check[..colon].trim());
                break;
            }
            break;
        }

        if let Some(key) = first_key {
            if key != "name" {
                return vec![MatchResult::new(
                    self.id(),
                    format!("The 'name' key should be first in a task, but found '{key}' first"),
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
        KeyOrderRule.check_task(&task, &file)
    }

    #[test]
    fn name_first_ok() {
        assert!(lint_task("- name: Task\n  debug:\n    msg: hi\n").is_empty());
    }

    #[test]
    fn module_first_flagged() {
        let r = lint_task("- debug:\n    msg: hi\n  name: Task\n");
        assert_eq!(r.len(), 1);
    }
}
