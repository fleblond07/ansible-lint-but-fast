use regex::Regex;
use std::sync::OnceLock;

use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

static EMPTY_STR_RE: OnceLock<Regex> = OnceLock::new();
fn empty_str_re() -> &'static Regex {
    EMPTY_STR_RE.get_or_init(|| {
        Regex::new(r#"==\s*['"]{2}|['"]{2}\s*==|!=\s*['"]{2}|['"]{2}\s*!="#).unwrap()
    })
}

/// `when:` conditions should not compare to empty string; use `var | length > 0`.
/// Rule ID: empty-string-compare
pub struct EmptyStringCompareRule;

impl Rule for EmptyStringCompareRule {
    fn id(&self) -> &str { "empty-string-compare" }
    fn description(&self) -> &str { "Don't compare to empty string; use 'var | length > 0' or '| bool'" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/empty-string-compare/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        for key in ["when", "failed_when", "changed_when"] {
            if let Some(val) = task.raw.get(key) {
                let s = match val.as_str() {
                    Some(s) => s,
                    None => continue,
                };
                if empty_str_re().is_match(s) {
                    return vec![MatchResult::new(
                        self.id(),
                        format!("Avoid comparing to empty string in '{key}'; use 'var | length > 0' or 'var is defined'"),
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
        EmptyStringCompareRule.check_task(&task, &file)
    }

    #[test]
    fn empty_string_compare_flagged() {
        let r = lint_task("- name: T\n  debug:\n    msg: hi\n  when: \"myvar == ''\"\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn non_empty_compare_ok() {
        assert!(lint_task("- name: T\n  debug:\n    msg: hi\n  when: myvar is defined\n").is_empty());
    }
}
