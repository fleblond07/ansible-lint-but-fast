use regex::Regex;
use std::sync::OnceLock;

use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

// Bare variable pattern: a plain word used where a Jinja2 `{{ var }}` expression is expected.
// Detect strings that look like bare variable references (no braces, not a path, not a number).
static BARE_VAR_RE: OnceLock<Regex> = OnceLock::new();
fn bare_var_re() -> &'static Regex {
    BARE_VAR_RE.get_or_init(|| {
        // A bare variable: alphanumeric + underscores only, not containing dots/slashes,
        // and not a YAML boolean or null.
        Regex::new(r"^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*$").unwrap()
    })
}

const YAML_KEYWORDS: &[&str] = &[
    "true", "false", "yes", "no", "on", "off", "null", "~",
    "True", "False", "Yes", "No", "On", "Off", "Null",
    "TRUE", "FALSE", "YES", "NO", "ON", "OFF", "NULL",
];

/// Detect deprecated bare variable references in `loop` / `with_items`.
/// Rule ID: deprecated-bare-vars
pub struct DeprecatedBareVarsRule;

impl Rule for DeprecatedBareVarsRule {
    fn id(&self) -> &str { "deprecated-bare-vars" }
    fn description(&self) -> &str { "Variables in loop/with_items must use Jinja2 syntax" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/deprecated-bare-vars/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["deprecated"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let loop_val = task.raw.get("loop").or_else(|| task.raw.get("with_items"));

        if let Some(val) = loop_val {
            if let Some(s) = val.as_str() {
                if bare_var_re().is_match(s) && !YAML_KEYWORDS.contains(&s.trim()) {
                    return vec![MatchResult::new(
                        self.id(),
                        format!("Use '{{{{ {s} }}}}' instead of bare variable '{s}' in loop"),
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
        DeprecatedBareVarsRule.check_task(&task, &file)
    }

    #[test]
    fn bare_var_in_loop_flagged() {
        let r = lint_task("- name: Loop\n  debug:\n    msg: item\n  loop: my_list\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn jinja2_var_ok() {
        assert!(lint_task("- name: Loop\n  debug:\n    msg: item\n  loop: \"{{ my_list }}\"\n").is_empty());
    }
}
