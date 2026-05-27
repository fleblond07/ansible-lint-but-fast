use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Modules should use Fully Qualified Collection Names (FQCN).
/// Rule ID: fqcn[action]
pub struct FqcnActionRule;

// Short names of builtin modules that should be written with their FQCN.
// Common ones that have a non-FQCN alias.
const SHOULD_BE_FQCN: &[(&str, &str)] = &[
    ("apt", "ansible.builtin.apt"),
    ("yum", "ansible.builtin.yum"),
    ("dnf", "ansible.builtin.dnf"),
    ("copy", "ansible.builtin.copy"),
    ("template", "ansible.builtin.template"),
    ("file", "ansible.builtin.file"),
    ("service", "ansible.builtin.service"),
    ("command", "ansible.builtin.command"),
    ("shell", "ansible.builtin.shell"),
    ("script", "ansible.builtin.script"),
    ("raw", "ansible.builtin.raw"),
    ("get_url", "ansible.builtin.get_url"),
    ("uri", "ansible.builtin.uri"),
    ("debug", "ansible.builtin.debug"),
    ("assert", "ansible.builtin.assert"),
    ("fail", "ansible.builtin.fail"),
    ("set_fact", "ansible.builtin.set_fact"),
    ("include_vars", "ansible.builtin.include_vars"),
    ("include_tasks", "ansible.builtin.include_tasks"),
    ("import_tasks", "ansible.builtin.import_tasks"),
    ("include_role", "ansible.builtin.include_role"),
    ("import_role", "ansible.builtin.import_role"),
    ("meta", "ansible.builtin.meta"),
    ("ping", "ansible.builtin.ping"),
    ("stat", "ansible.builtin.stat"),
    ("find", "ansible.builtin.find"),
    ("lineinfile", "ansible.builtin.lineinfile"),
    ("blockinfile", "ansible.builtin.blockinfile"),
    ("replace", "ansible.builtin.replace"),
    ("user", "ansible.builtin.user"),
    ("group", "ansible.builtin.group"),
    ("cron", "ansible.builtin.cron"),
    ("git", "ansible.builtin.git"),
    ("pip", "ansible.builtin.pip"),
    ("package", "ansible.builtin.package"),
    ("synchronize", "ansible.posix.synchronize"),
    ("sysctl", "ansible.posix.sysctl"),
    ("firewalld", "ansible.posix.firewalld"),
    ("mount", "ansible.posix.mount"),
    ("acl", "ansible.posix.acl"),
    ("authorized_key", "ansible.posix.authorized_key"),
    ("setup", "ansible.builtin.setup"),
    ("gather_facts", "ansible.builtin.gather_facts"),
    ("wait_for", "ansible.builtin.wait_for"),
    ("pause", "ansible.builtin.pause"),
    ("unarchive", "ansible.builtin.unarchive"),
    ("archive", "community.general.archive"),
];

impl Rule for FqcnActionRule {
    fn id(&self) -> &str { "fqcn[action]" }
    fn description(&self) -> &str { "Use FQCN for module names" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/fqcn/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        if let Some(module) = &task.module {
            if let Some((_, fqcn)) = SHOULD_BE_FQCN.iter().find(|(short, _)| short == module) {
                return vec![MatchResult::new(
                    self.id(),
                    format!("Use FQCN '{fqcn}' instead of '{module}'"),
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
        FqcnActionRule.check_task(&task, &file)
    }

    #[test]
    fn fqcn_ok() {
        assert!(lint_task("- name: Debug\n  ansible.builtin.debug:\n    msg: hi\n").is_empty());
    }

    #[test]
    fn short_name_flagged() {
        let r = lint_task("- name: Debug\n  debug:\n    msg: hi\n");
        assert_eq!(r.len(), 1);
        assert!(r[0].message.contains("ansible.builtin.debug"));
    }
}
