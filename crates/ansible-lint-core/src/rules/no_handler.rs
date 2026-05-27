use crate::parser::task::Task;
use crate::registry::Profile;
use crate::rule::{LintFile, MatchResult, Rule, Severity};

/// Tasks that restart/reload services should use handlers instead of direct tasks.
/// Rule ID: no-handler
pub struct NoHandlerRule;

const SERVICE_ACTIONS: &[&str] = &["restarted", "reloaded"];
const SERVICE_MODULES: &[&str] = &[
    "service", "systemd", "ansible.builtin.service", "ansible.builtin.systemd",
];

impl Rule for NoHandlerRule {
    fn id(&self) -> &str { "no-handler" }
    fn description(&self) -> &str { "Tasks that restart services should use notify/handlers" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/no-handler/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Moderate] }

    fn check_task(&self, task: &Task, file: &LintFile) -> Vec<MatchResult> {
        use crate::parser::task::ModuleArgs;

        let module = match &task.module {
            Some(m) => m.as_str(),
            None => return vec![],
        };
        if !SERVICE_MODULES.contains(&module) {
            return vec![];
        }

        // Only flag restart/reload tasks that are NOT already in a handler block.
        // Heuristic: if the file is a handlers file, skip it.
        if file.kind == crate::discovery::FileKind::Handlers {
            return vec![];
        }

        let state = match &task.module_args {
            Some(ModuleArgs::Mapping(args)) => args.get("state").and_then(|v| v.as_str()).unwrap_or(""),
            _ => "",
        };

        if SERVICE_ACTIONS.contains(&state) {
            // If task already has a when: clause, it might be intentional.
            if task.raw.contains_key("when") {
                return vec![];
            }
            vec![MatchResult::new(
                self.id(),
                format!("Service task with state '{state}' should be a handler triggered via notify"),
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

    fn lint_task_kind(yaml: &str, kind: FileKind) -> Vec<MatchResult> {
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        let file = LintFile { path: PathBuf::from("test.yml"), content: yaml.to_string(), kind };
        NoHandlerRule.check_task(&task, &file)
    }

    #[test]
    fn restart_in_tasks_flagged() {
        let r = lint_task_kind(
            "- name: Restart nginx\n  service:\n    name: nginx\n    state: restarted\n",
            FileKind::Tasks,
        );
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn restart_in_handlers_ok() {
        let r = lint_task_kind(
            "- name: Restart nginx\n  service:\n    name: nginx\n    state: restarted\n",
            FileKind::Handlers,
        );
        assert!(r.is_empty());
    }

    #[test]
    fn service_started_ok() {
        let r = lint_task_kind(
            "- name: Start nginx\n  service:\n    name: nginx\n    state: started\n",
            FileKind::Tasks,
        );
        assert!(r.is_empty());
    }
}
