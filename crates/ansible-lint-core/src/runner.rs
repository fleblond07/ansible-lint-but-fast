use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::discovery::discover_files;
use crate::error::LintError;
use crate::parser::playbook::{all_tasks, parse_playbook};
use crate::parser::task::parse_task;
use crate::parser::yaml::parse_yaml_with_positions;
use crate::registry::RuleRegistry;
use crate::rule::{LintFile, MatchResult, Severity};

/// Pairs of (file_path, rule_id) that should be suppressed.
pub type IgnoreSet = HashSet<(PathBuf, String)>;

/// Load `.ansible-lint-ignore` or `.config/ansible-lint-ignore.txt`.
pub fn load_ignore_file(project_dir: &Path) -> IgnoreSet {
    let candidates = [
        project_dir.join(".ansible-lint-ignore"),
        project_dir.join(".config").join("ansible-lint-ignore.txt"),
    ];

    let mut set = HashSet::new();
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let mut parts = line.splitn(2, ' ');
                if let (Some(file), Some(rule)) = (parts.next(), parts.next()) {
                    set.insert((PathBuf::from(file.trim()), rule.trim().to_string()));
                }
            }
        }
    }
    set
}

/// Build a map of line_number → set of rule IDs suppressed by `# noqa` comments.
/// `# noqa` with no rule list suppresses all rules on that line.
/// `# noqa: rule1,rule2` suppresses specific rules.
pub fn build_noqa_map(content: &str) -> HashMap<usize, Option<Vec<String>>> {
    let mut map = HashMap::new();
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        if let Some(noqa_pos) = line.find("# noqa") {
            let after = line[noqa_pos + 6..].trim();
            if after.is_empty() || after.starts_with('\n') {
                // Suppress all rules on this line.
                map.insert(line_num, None);
            } else if let Some(rest) = after.strip_prefix(':') {
                // Suppress specific rules.
                let rules: Vec<String> = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                map.insert(line_num, Some(rules));
            } else {
                // Bare `# noqa` without colon.
                map.insert(line_num, None);
            }
        }
    }
    map
}

/// Check whether a match is suppressed by noqa.
fn is_noqa_suppressed(
    m: &MatchResult,
    noqa_map: &HashMap<usize, Option<Vec<String>>>,
) -> bool {
    match noqa_map.get(&m.location.line) {
        Some(None) => true, // suppress all
        Some(Some(rules)) => rules.iter().any(|r| r == &m.rule_id || m.tag.as_deref() == Some(r.as_str())),
        None => false,
    }
}

pub struct LintRunner<'a> {
    pub config: &'a Config,
    pub registry: &'a RuleRegistry,
    pub project_dir: PathBuf,
}

impl<'a> LintRunner<'a> {
    pub fn new(config: &'a Config, registry: &'a RuleRegistry, project_dir: PathBuf) -> Self {
        Self { config, registry, project_dir }
    }

    pub fn run(&self, input_paths: &[PathBuf]) -> Result<Vec<MatchResult>, LintError> {
        let active_rules = self.registry.active_rules(
            self.config.profile,
            &self.config.skip_list,
            &self.config.enable_list,
        );

        let files = discover_files(input_paths, &self.config.exclude_paths)?;
        let ignore_set = load_ignore_file(&self.project_dir);

        let mut results: Vec<MatchResult> = Vec::new();

        for disc_file in &files {
            let content = match std::fs::read_to_string(&disc_file.path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: could not read {:?}: {e}", disc_file.path);
                    continue;
                }
            };

            let noqa_map = build_noqa_map(&content);

            let lint_file = LintFile {
                path: disc_file.path.clone(),
                content: content.clone(),
                kind: disc_file.kind.clone(),
            };

            // 1. Raw-file checks (yaml rules, etc.).
            for rule in &active_rules {
                let mut matches = rule.check_raw_file(&lint_file);
                self.apply_warn_list(&mut matches);
                // Filter noqa inline suppressions.
                matches.retain(|m| !is_noqa_suppressed(m, &noqa_map));
                results.extend(matches);
            }

            // 2. Parse YAML.
            let docs = match parse_yaml_with_positions(&content, &disc_file.path.to_string_lossy()) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Warning: YAML parse error in {:?}: {e}", disc_file.path);
                    continue;
                }
            };

            if docs.is_empty() {
                continue;
            }
            let doc = &docs[0];

            // 3. Try to parse as playbook.
            if let Some(plays) = parse_playbook(doc) {
                for play in &plays {
                    for rule in &active_rules {
                        let mut matches = rule.check_play(play, &lint_file);
                        self.apply_warn_list(&mut matches);
                        matches.retain(|m| !is_noqa_suppressed(m, &noqa_map));
                        results.extend(matches);
                    }
                    for task in all_tasks(play) {
                        for rule in &active_rules {
                            let mut matches = rule.check_task(task, &lint_file);
                            self.apply_warn_list(&mut matches);
                            matches.retain(|m| !is_noqa_suppressed(m, &noqa_map));
                            results.extend(matches);
                        }
                    }
                }
            } else {
                // Parse as task list.
                if let Some(items) = doc.as_vec() {
                    for item in items {
                        if let Some(task) = parse_task(item) {
                            for rule in &active_rules {
                                let mut matches = rule.check_task(&task, &lint_file);
                                self.apply_warn_list(&mut matches);
                                matches.retain(|m| !is_noqa_suppressed(m, &noqa_map));
                                results.extend(matches);
                            }
                        }
                    }
                }
            }
        }

        // Filter .ansible-lint-ignore suppressions.
        results.retain(|m| {
            let rel = m.filename
                .strip_prefix(&self.project_dir)
                .unwrap_or(&m.filename);
            !ignore_set.contains(&(rel.to_path_buf(), m.rule_id.clone()))
        });

        // Sort for deterministic output: filename → line → col → rule.
        results.sort_by(|a, b| {
            a.filename.cmp(&b.filename)
                .then(a.location.line.cmp(&b.location.line))
                .then(a.location.column.cmp(&b.location.column))
                .then(a.rule_id.cmp(&b.rule_id))
        });

        Ok(results)
    }

    fn apply_warn_list(&self, results: &mut [MatchResult]) {
        for m in results.iter_mut() {
            if self.config.warn_list.iter().any(|w| w == &m.rule_id || m.tags().contains(&w.as_str())) {
                m.severity = Severity::Warning;
            }
        }
    }
}

// Extend MatchResult to expose tags for warn-list checks.
impl MatchResult {
    fn tags(&self) -> Vec<&str> {
        self.tag.as_deref().map(|t| vec![t]).unwrap_or_default()
    }
}

/// Count errors (non-warning, non-info results).
pub fn count_errors(results: &[MatchResult], strict: bool) -> usize {
    results.iter().filter(|m| {
        if strict {
            m.severity >= Severity::Warning
        } else {
            m.severity == Severity::Error
        }
    }).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noqa_all_suppresses() {
        let content = "enabled: yes  # noqa\n";
        let map = build_noqa_map(content);
        assert!(map.contains_key(&1));
        assert!(map[&1].is_none());
    }

    #[test]
    fn noqa_specific_rule() {
        let content = "enabled: yes  # noqa: yaml[truthy]\n";
        let map = build_noqa_map(content);
        assert_eq!(map[&1].as_ref().unwrap(), &["yaml[truthy]"]);
    }

    #[test]
    fn noqa_multiple_rules() {
        let content = "task: foo  # noqa: rule1,rule2\n";
        let map = build_noqa_map(content);
        let rules = map[&1].as_ref().unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn no_noqa_comment() {
        let content = "enabled: yes\n";
        let map = build_noqa_map(content);
        assert!(map.is_empty());
    }
}
