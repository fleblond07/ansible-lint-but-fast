use std::collections::HashMap;

use crate::parser::yaml::MarkedNode;
use crate::rule::Location;

/// Represents a single Ansible task or handler.
#[derive(Debug, Clone)]
pub struct Task {
    pub name: Option<String>,
    pub module: Option<String>,
    pub module_args: Option<ModuleArgs>,
    pub r#become: Option<bool>,
    pub become_user: Option<String>,
    /// `changed_when` can be a string expression or a boolean.
    pub changed_when: Option<String>,
    pub changed_when_bool: Option<bool>,
    pub no_log: Option<bool>,
    pub loop_var: Option<String>,
    pub notify: Option<Vec<String>>,
    /// Raw key-value pairs from the task mapping, for rules that need full access.
    pub raw: HashMap<String, RawValue>,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum ModuleArgs {
    /// Free-form string argument (e.g. `command: echo hello`).
    FreeForm(String),
    /// Structured mapping.
    Mapping(HashMap<String, RawValue>),
}

#[derive(Debug, Clone)]
pub enum RawValue {
    String(String),
    Bool(bool),
    Int(i64),
    List(Vec<RawValue>),
    Map(HashMap<String, RawValue>),
    Null,
}

impl RawValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RawValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RawValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, RawValue>> {
        match self {
            RawValue::Map(m) => Some(m),
            _ => None,
        }
    }
}

// YAML keys that are Ansible task directives (not module names).
pub const TASK_DIRECTIVES: &[&str] = &[
    "name", "when", "loop", "with_items", "with_list", "with_dict",
    "with_fileglob", "with_first_found", "with_indexed_items",
    "with_nested", "with_subelements", "with_together", "with_random_choice",
    "with_sequence", "with_flattened", "register", "ignore_errors",
    "no_log", "become", "become_user", "become_method", "become_flags",
    "changed_when", "failed_when", "notify", "tags", "vars",
    "environment", "delegate_to", "delegate_facts", "run_once",
    "any_errors_fatal", "check_mode", "diff", "timeout",
    "module_defaults", "collections", "listen", "block", "rescue", "always",
    "loop_var", "loop_control", "include_role", "include_tasks",
    "import_role", "import_tasks", "include", "import_playbook",
    "meta",
];

/// Parse a task from a `MarkedNode` mapping.
pub fn parse_task(node: &MarkedNode) -> Option<Task> {
    let pairs = node.as_hash()?;
    let location = node.location.clone();

    let mut name: Option<String> = None;
    let mut module: Option<String> = None;
    let mut module_args: Option<ModuleArgs> = None;
    let mut r#become: Option<bool> = None;
    let mut become_user: Option<String> = None;
    let mut changed_when: Option<String> = None;
    let mut changed_when_bool: Option<bool> = None;
    let mut no_log: Option<bool> = None;
    let mut loop_var: Option<String> = None;
    let mut raw: HashMap<String, RawValue> = HashMap::new();

    for (key_node, val_node) in pairs {
        let key = key_node.as_str()?;
        let raw_val = node_to_raw(val_node);

        raw.insert(key.to_string(), raw_val.clone());

        match key {
            "name" => name = val_node.as_str().map(str::to_string),
            "become" => r#become = val_node.as_bool(),
            "become_user" => become_user = val_node.as_str().map(str::to_string),
            "changed_when" => {
                changed_when = val_node.as_str().map(str::to_string);
                changed_when_bool = val_node.as_bool();
            }
            "no_log" => no_log = val_node.as_bool(),
            "loop_var" => loop_var = val_node.as_str().map(str::to_string),
            _ if !TASK_DIRECTIVES.contains(&key) => {
                // This key is likely the module name.
                module = Some(key.to_string());
                module_args = Some(match &raw_val {
                    RawValue::String(s) => ModuleArgs::FreeForm(s.clone()),
                    RawValue::Map(m) => ModuleArgs::Mapping(m.clone()),
                    _ => ModuleArgs::FreeForm(String::new()),
                });
            }
            _ => {}
        }
    }

    Some(Task {
        name,
        module,
        module_args,
        r#become,
        become_user,
        changed_when,
        changed_when_bool,
        no_log,
        loop_var,
        notify: None,
        raw,
        location,
    })
}

fn node_to_raw(node: &MarkedNode) -> RawValue {
    use crate::parser::yaml::MarkedValue;
    match &node.value {
        MarkedValue::String(s) => RawValue::String(s.clone()),
        MarkedValue::Boolean(b) => RawValue::Bool(*b),
        MarkedValue::Integer(n) => RawValue::Int(*n),
        MarkedValue::Null => RawValue::Null,
        MarkedValue::Array(items) => RawValue::List(items.iter().map(node_to_raw).collect()),
        MarkedValue::Hash(pairs) => {
            let mut map = HashMap::new();
            for (k, v) in pairs {
                if let Some(key) = k.as_str() {
                    map.insert(key.to_string(), node_to_raw(v));
                }
            }
            RawValue::Map(map)
        }
        MarkedValue::Real(f) => RawValue::String(f.to_string()),
        MarkedValue::BadValue => RawValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::yaml::parse_yaml_with_positions;

    #[test]
    fn test_parse_named_task() {
        let yaml = "- name: Install nginx\n  apt:\n    name: nginx\n    state: present\n";
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        assert_eq!(task.name.as_deref(), Some("Install nginx"));
        assert_eq!(task.module.as_deref(), Some("apt"));
    }

    #[test]
    fn test_parse_anonymous_task() {
        let yaml = "- debug:\n    msg: hello\n";
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let items = docs[0].as_vec().unwrap();
        let task = parse_task(&items[0]).unwrap();
        assert!(task.name.is_none());
        assert_eq!(task.module.as_deref(), Some("debug"));
    }
}
