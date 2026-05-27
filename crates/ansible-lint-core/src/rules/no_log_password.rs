use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

const SENSITIVE_KEYS: &[&str] = &[
    "password", "passwd", "secret", "token", "api_key", "private_key",
    "access_key", "secret_key", "auth_token", "auth_pass", "vault_password",
];

/// Tasks that use password-like parameters must have `no_log: true`.
/// Rule ID: no-log-password
pub struct NoLogPasswordRule;

impl Rule for NoLogPasswordRule {
    fn id(&self) -> &str { "no-log-password" }
    fn description(&self) -> &str { "Tasks that deal with passwords must have no_log enabled" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-log-password/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["security"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        // Already has no_log: true — fine.
        if task.no_log == Some(true) {
            return vec![];
        }

        let has_sensitive = task.raw.iter().any(|(k, _)| {
            let k_lower = k.to_lowercase();
            SENSITIVE_KEYS.iter().any(|&s| k_lower.contains(s))
        }) || task.module_args.as_ref().is_some_and(|args| {
            if let crate::parser::task::ModuleArgs::Mapping(m) = args {
                m.keys().any(|k| {
                    let k_lower = k.to_lowercase();
                    SENSITIVE_KEYS.iter().any(|&s| k_lower.contains(s))
                })
            } else {
                false
            }
        });

        if has_sensitive {
            vec![MatchResult::new(
                self.id(),
                "Task uses a sensitive parameter; add 'no_log: true'",
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
        NoLogPasswordRule.check_task(&task, &file)
    }

    #[test]
    fn password_without_no_log_flagged() {
        let r = lint_task("- name: Create user\n  user:\n    name: bob\n    password: secret123\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn password_with_no_log_ok() {
        let r = lint_task("- name: Create user\n  user:\n    name: bob\n    password: secret123\n  no_log: true\n");
        assert!(r.is_empty());
    }

    #[test]
    fn no_sensitive_keys_ok() {
        assert!(lint_task("- name: Debug\n  debug:\n    msg: hello\n").is_empty());
    }
}
