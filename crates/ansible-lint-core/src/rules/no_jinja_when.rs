use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// `when:` clauses should not use Jinja2 `{{ }}` braces — they are implicit.
/// Rule ID: no-jinja-when
pub struct NoJinjaWhenRule;

impl Rule for NoJinjaWhenRule {
    fn id(&self) -> &str { "no-jinja-when" }
    fn description(&self) -> &str { "when: conditions should not use {{ }} Jinja2 syntax" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-jinja-when/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["syntax"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if let Some(when_val) = task.raw.get("when") {
            let when_str = match when_val.as_str() {
                Some(s) => s.to_string(),
                None => return vec![],
            };
            let trimmed = when_str.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                return vec![MatchResult::new(
                    self.id(),
                    format!("when: clause '{trimmed}' should not use Jinja2 braces; use bare expression"),
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
        NoJinjaWhenRule.check_task(&task, &file)
    }

    #[test]
    fn bare_when_ok() {
        assert!(lint_task("- name: Task\n  debug:\n    msg: hi\n  when: my_var is defined\n").is_empty());
    }

    #[test]
    fn jinja_when_flagged() {
        let r = lint_task("- name: Task\n  debug:\n    msg: hi\n  when: \"{{ my_var }}\"\n");
        assert_eq!(r.len(), 1);
    }
}
