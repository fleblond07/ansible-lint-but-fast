use regex::Regex;
use std::sync::OnceLock;

use crate::parser::task::{ModuleArgs, Task};
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Use dedicated modules instead of `command`/`shell` where possible.
/// Rule ID: command-instead-of-module
pub struct CommandInsteadOfModuleRule;

/// Use `command` instead of `shell` when no shell features are needed.
/// Rule ID: command-instead-of-shell
pub struct CommandInsteadOfShellRule;

// (regex pattern for command string, suggested module name)
static MODULE_PATTERNS: &[(&str, &str)] = &[
    (r"^\s*apt(-get)?\s+(install|remove|update|upgrade|purge)\b", "apt"),
    (r"^\s*yum\s+(install|remove|update|erase)\b", "yum"),
    (r"^\s*dnf\s+(install|remove|update|erase)\b", "dnf"),
    (r"^\s*pip\s+install\b", "pip"),
    (r"^\s*systemctl\s+(start|stop|restart|reload|enable|disable)\b", "service/systemd"),
    (r"^\s*service\s+\w+\s+(start|stop|restart)\b", "service"),
    (r"^\s*chmod\b", "file"),
    (r"^\s*chown\b", "file"),
    (r"^\s*mkdir\b", "file"),
    (r"^\s*rm\s+-rf?\b", "file"),
    (r"^\s*cp\b", "copy"),
    (r"^\s*curl\b", "get_url/uri"),
    (r"^\s*wget\b", "get_url"),
    (r"^\s*git\s+(clone|pull|push)\b", "git"),
    (r"^\s*useradd\b", "user"),
    (r"^\s*groupadd\b", "group"),
    (r"^\s*tar\b", "unarchive"),
    (r"^\s*unzip\b", "unarchive"),
];

// Shell-specific features indicating shell is genuinely needed.
static SHELL_FEATURES_RE: OnceLock<Regex> = OnceLock::new();

fn shell_features_re() -> &'static Regex {
    SHELL_FEATURES_RE.get_or_init(|| {
        Regex::new(r#"[|&;<>$`!*?\[\]{}'"]|\b(if|for|while|until|case|do|done|fi|esac|then|else|elif)\b"#).unwrap()
    })
}

impl Rule for CommandInsteadOfModuleRule {
    fn id(&self) -> &str { "command-instead-of-module" }
    fn description(&self) -> &str { "Avoid using command when a dedicated module is available" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/command-instead-of-module/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["command-shell", "idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if module != "command" && module != "ansible.builtin.command"
            && module != "shell" && module != "ansible.builtin.shell" {
            return vec![];
        }

        let cmd = extract_cmd(task);

        for (pattern, suggestion) in MODULE_PATTERNS {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&cmd) {
                return vec![MatchResult::new(
                    self.id(),
                    format!("Use the '{suggestion}' module instead of running '{}'", module),
                    file.path.clone(),
                    task.location.clone(),
                    self.severity(),
                )];
            }
        }

        vec![]
    }
}

impl Rule for CommandInsteadOfShellRule {
    fn id(&self) -> &str { "command-instead-of-shell" }
    fn description(&self) -> &str { "Use command instead of shell when shell features are not needed" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/command-instead-of-module/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["command-shell", "idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if module != "shell" && module != "ansible.builtin.shell" {
            return vec![];
        }

        let cmd = extract_cmd(task);
        if shell_features_re().is_match(&cmd) {
            return vec![];
        }

        vec![MatchResult::new(
            self.id(),
            "Use 'command' instead of 'shell' when shell features are not needed",
            file.path.clone(),
            task.location.clone(),
            self.severity(),
        )]
    }
}

fn extract_cmd(task: &Task) -> String {
    match &task.module_args {
        Some(ModuleArgs::FreeForm(s)) => s.clone(),
        Some(ModuleArgs::Mapping(m)) => m.get("cmd")
            .or_else(|| m.get("_raw_params"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use crate::parser::yaml::parse_yaml_with_positions;
    use crate::parser::task::parse_task;
    use std::path::PathBuf;

    fn lint_module(yaml: &str) -> Vec<MatchResult> {
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        let file = LintFile { path: PathBuf::from("test.yml"), content: yaml.to_string(), kind: FileKind::Tasks };
        CommandInsteadOfModuleRule.check_task(&task, &file)
    }

    fn lint_shell(yaml: &str) -> Vec<MatchResult> {
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        let file = LintFile { path: PathBuf::from("test.yml"), content: yaml.to_string(), kind: FileKind::Tasks };
        CommandInsteadOfShellRule.check_task(&task, &file)
    }

    #[test]
    fn apt_via_command_flagged() {
        let r = lint_module("- name: Install pkg\n  command: apt-get install -y nginx\n");
        assert_eq!(r.len(), 1);
        assert!(r[0].message.contains("apt"));
    }

    #[test]
    fn shell_without_features_flagged() {
        let r = lint_shell("- name: Run\n  shell: echo hello\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn shell_with_pipe_ok() {
        let r = lint_shell("- name: Pipe\n  shell: cat file | grep foo\n");
        assert!(r.is_empty());
    }
}
