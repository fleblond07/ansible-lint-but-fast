use regex::Regex;
use std::sync::OnceLock;

use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

static LITERAL_CMP_RE: OnceLock<Regex> = OnceLock::new();
fn literal_cmp_re() -> &'static Regex {
    LITERAL_CMP_RE.get_or_init(|| {
        // Matches: `expr == True/False/None` or `True/False/None == expr`
        Regex::new(r"(?:==|!=|is)\s+(?:True|False|None)\b|(?:True|False|None)\b\s+(?:==|!=|is)").unwrap()
    })
}

/// `when:` conditions should not compare to Python literals True/False/None.
/// Rule ID: literal-compare
pub struct LiteralCompareRule;

impl Rule for LiteralCompareRule {
    fn id(&self) -> &str { "literal-compare" }
    fn description(&self) -> &str { "Use 'true'/'false' not 'True'/'False' in conditions" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/literal-compare/" }
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
                if literal_cmp_re().is_match(s) {
                    return vec![MatchResult::new(
                        self.id(),
                        format!("Use Jinja2 boolean/null literals (true/false/none) not Python literals in '{key}'"),
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
        LiteralCompareRule.check_task(&task, &file)
    }

    #[test]
    fn true_compare_flagged() {
        // Quoted to ensure YAML treats it as a string (unquoted True is parsed as bool).
        let r = lint_task("- name: T\n  debug:\n    msg: hi\n  when: \"result == True\"\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn lowercase_true_ok() {
        assert!(lint_task("- name: T\n  debug:\n    msg: hi\n  when: result == true\n").is_empty());
    }
}
