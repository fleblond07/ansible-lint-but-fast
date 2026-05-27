use std::collections::HashSet;

use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// YAML mappings must not have duplicate keys.
/// Rule ID: yaml[key-duplicates]
pub struct YamlKeyDuplicatesRule;

impl Rule for YamlKeyDuplicatesRule {
    fn id(&self) -> &str { "yaml[key-duplicates]" }
    fn description(&self) -> &str { "Duplicate keys in YAML mappings are not allowed" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["yaml"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Min] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();
        // Stack of (indent_level, seen_keys_at_level)
        let mut stack: Vec<(usize, HashSet<String>)> = Vec::new();
        // Track the current indent level.
        // (unused: previously tracked for future nested block support)

        for (i, line) in file.content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" || trimmed == "..." {
                continue;
            }

            let indent = line.len() - trimmed.len();

            // Pop stack levels strictly deeper than current indent.
            while let Some(&(level, _)) = stack.last() {
                if level > indent {
                    stack.pop();
                } else {
                    break;
                }
            }

            // Check if this line is a mapping key.
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();
                // Skip list items.
                if key.starts_with('-') {
                    continue;
                }
                // Ensure we have a level entry.
                if stack.last().is_none_or(|&(l, _)| l != indent) {
                    stack.push((indent, HashSet::new()));
                }
                if let Some((_, ref mut seen)) = stack.last_mut() {
                    if !seen.insert(key.clone()) {
                        results.push(MatchResult::new(
                            self.id(),
                            format!("Duplicate key '{key}' found in YAML mapping"),
                            file.path.clone(),
                            Location { line: i + 1, column: indent + 1 },
                            self.severity(),
                        ));
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use std::path::PathBuf;

    fn lint(content: &str) -> Vec<MatchResult> {
        let file = LintFile { path: PathBuf::from("t.yml"), content: content.to_string(), kind: FileKind::Tasks };
        YamlKeyDuplicatesRule.check_raw_file(&file)
    }

    #[test]
    fn unique_keys_ok() { assert!(lint("---\nfoo: 1\nbar: 2\n").is_empty()); }

    #[test]
    fn duplicate_key_flagged() {
        let r = lint("---\nfoo: 1\nfoo: 2\n");
        assert_eq!(r.len(), 1);
        assert!(r[0].message.contains("foo"));
    }
}
