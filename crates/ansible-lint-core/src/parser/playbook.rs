use crate::parser::task::{parse_task, Task};
use crate::parser::yaml::MarkedNode;
use crate::rule::Location;

/// A single play in a playbook.
#[derive(Debug, Clone)]
pub struct Play {
    pub name: Option<String>,
    pub hosts: Option<String>,
    pub r#become: Option<bool>,
    pub become_user: Option<String>,
    pub tasks: Vec<Task>,
    pub handlers: Vec<Task>,
    pub pre_tasks: Vec<Task>,
    pub post_tasks: Vec<Task>,
    pub vars: Vec<(String, MarkedNode)>,
    pub location: Location,
}

/// Parse a list of plays from the top-level YAML document.
/// Returns `None` if the document is not a playbook (list of plays with `hosts`).
pub fn parse_playbook(doc: &MarkedNode) -> Option<Vec<Play>> {
    let items = doc.as_vec()?;

    // A playbook is a list where at least the first item has a `hosts` key.
    let first = items.first()?;
    if first.get("hosts").is_none() && first.get("import_playbook").is_none() {
        return None;
    }

    let plays = items.iter().filter_map(parse_play).collect();
    Some(plays)
}

fn parse_play(node: &MarkedNode) -> Option<Play> {
    let location = node.location.clone();

    let name = node.get("name").and_then(|n| n.as_str()).map(str::to_string);
    let hosts = node.get("hosts").and_then(|n| n.as_str()).map(str::to_string);
    let r#become = node.get("become").and_then(|n| n.as_bool());
    let become_user = node.get("become_user").and_then(|n| n.as_str()).map(str::to_string);

    let tasks = parse_task_list(node.get("tasks"));
    let handlers = parse_task_list(node.get("handlers"));
    let pre_tasks = parse_task_list(node.get("pre_tasks"));
    let post_tasks = parse_task_list(node.get("post_tasks"));

    let vars = node.get("vars")
        .and_then(|n| n.as_hash())
        .map(|pairs| {
            pairs.iter()
                .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), v.clone())))
                .collect()
        })
        .unwrap_or_default();

    Some(Play {
        name,
        hosts,
        r#become,
        become_user,
        tasks,
        handlers,
        pre_tasks,
        post_tasks,
        vars,
        location,
    })
}

fn parse_task_list(node: Option<&MarkedNode>) -> Vec<Task> {
    node.and_then(|n| n.as_vec())
        .map(|items| items.iter().filter_map(parse_task).collect())
        .unwrap_or_default()
}

/// Return all tasks in a play (tasks + pre_tasks + post_tasks + handlers).
pub fn all_tasks(play: &Play) -> Vec<&Task> {
    play.tasks.iter()
        .chain(play.pre_tasks.iter())
        .chain(play.post_tasks.iter())
        .chain(play.handlers.iter())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::yaml::parse_yaml_with_positions;

    #[test]
    fn test_parse_playbook() {
        let yaml = r#"
- name: Test play
  hosts: all
  tasks:
    - name: Say hello
      debug:
        msg: hello
"#;
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        let plays = parse_playbook(&docs[0]).unwrap();
        assert_eq!(plays.len(), 1);
        assert_eq!(plays[0].name.as_deref(), Some("Test play"));
        assert_eq!(plays[0].tasks.len(), 1);
    }

    #[test]
    fn test_non_playbook_returns_none() {
        let yaml = "- name: bare task\n  debug:\n    msg: hi\n";
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        assert!(parse_playbook(&docs[0]).is_none());
    }
}
