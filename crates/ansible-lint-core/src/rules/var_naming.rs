use regex::Regex;
use std::sync::OnceLock;

use crate::parser::task::{ModuleArgs, Task};
use crate::parser::playbook::Play;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const DEFAULT_PATTERN: &str = r"^[a-z_][a-z0-9_]*$";

static VAR_RE: OnceLock<Regex> = OnceLock::new();
fn var_re() -> &'static Regex {
    VAR_RE.get_or_init(|| Regex::new(DEFAULT_PATTERN).unwrap())
}

/// Variable names must match the configured pattern.
/// Rule ID: var-naming[pattern]
pub struct VarNamingRule;

impl Rule for VarNamingRule {
    fn id(&self) -> &str { "var-naming[pattern]" }
    fn description(&self) -> &str { "Variables should use snake_case naming" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/var-naming/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        // Check `register` variable name.
        if let Some(reg_val) = task.raw.get("register") {
            if let Some(var_name) = reg_val.as_str() {
                if !var_re().is_match(var_name) {
                    results.push(MatchResult::new(
                        self.id(),
                        format!("Variable name '{var_name}' does not match pattern '{DEFAULT_PATTERN}'"),
                        file.path.clone(),
                        task.location.clone(),
                        self.severity(),
                    ));
                }
            }
        }

        // Check `set_fact` keys.
        if task.module.as_deref() == Some("set_fact")
            || task.module.as_deref() == Some("ansible.builtin.set_fact")
        {
            if let Some(ModuleArgs::Mapping(args)) = &task.module_args {
                for key in args.keys() {
                    if !var_re().is_match(key) {
                        results.push(MatchResult::new(
                            self.id(),
                            format!("Variable name '{key}' does not match pattern '{DEFAULT_PATTERN}'"),
                            file.path.clone(),
                            task.location.clone(),
                            self.severity(),
                        ));
                    }
                }
            }
        }

        results
    }

    fn check_play(&self, play: &Play, file: &LintFile) -> Vec<MatchResult> {
        play.vars.iter()
            .filter_map(|(name, _)| {
                if !var_re().is_match(name) {
                    Some(MatchResult::new(
                        self.id(),
                        format!("Variable name '{name}' does not match pattern '{DEFAULT_PATTERN}'"),
                        file.path.clone(),
                        play.location.clone(),
                        self.severity(),
                    ))
                } else {
                    None
                }
            })
            .collect()
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
        VarNamingRule.check_task(&task, &file)
    }

    #[test]
    fn snake_case_ok() {
        assert!(lint_task("- name: Cmd\n  command: echo hi\n  register: my_result\n").is_empty());
    }

    #[test]
    fn camel_case_flagged() {
        let r = lint_task("- name: Cmd\n  command: echo hi\n  register: myResult\n");
        assert_eq!(r.len(), 1);
    }
}
