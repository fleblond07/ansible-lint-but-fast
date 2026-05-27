use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// YAML indentation must use spaces, not tabs.
/// Rule ID: yaml[indentation]
pub struct YamlIndentationRule;

impl Rule for YamlIndentationRule {
    fn id(&self) -> &str { "yaml[indentation]" }
    fn description(&self) -> &str { "YAML indentation must use spaces, not tabs" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        file.content
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                if line.starts_with('\t') || line.contains("\t ") || line.contains(" \t") {
                    let col = line.find('\t').unwrap_or(0) + 1;
                    Some(MatchResult::new(
                        self.id(),
                        "Tabs found in indentation; use spaces instead",
                        file.path.clone(),
                        Location { line: i + 1, column: col },
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
    use std::path::PathBuf;

    fn lint(content: &str) -> Vec<MatchResult> {
        let file = LintFile { path: PathBuf::from("t.yml"), content: content.to_string(), kind: FileKind::Tasks };
        YamlIndentationRule.check_raw_file(&file)
    }

    #[test]
    fn spaces_ok() { assert!(lint("  name: foo\n").is_empty()); }

    #[test]
    fn tab_flagged() {
        let r = lint("\tname: foo\n");
        assert_eq!(r.len(), 1);
    }
}
