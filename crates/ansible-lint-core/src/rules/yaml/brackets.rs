use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Flow sequences and mappings should have consistent spacing inside brackets.
/// Rule ID: yaml[brackets]
pub struct YamlBracketsRule;

impl Rule for YamlBracketsRule {
    fn id(&self) -> &str { "yaml[brackets]" }
    fn description(&self) -> &str { "Too many spaces inside brackets" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (i, line) in file.content.lines().enumerate() {
            if line.trim().starts_with('#') { continue; }

            let bytes = line.as_bytes();
            for (j, &b) in bytes.iter().enumerate() {
                // Check for `[ ` (space after opening bracket).
                if b == b'['
                    && bytes.get(j + 1) == Some(&b' ') && bytes.get(j + 2) != Some(&b']') {
                    results.push(MatchResult::new(
                        self.id(),
                        "Too many spaces inside brackets",
                        file.path.clone(),
                        Location { line: i + 1, column: j + 1 },
                        self.severity(),
                    ));
                }
                // Check for ` ]` (space before closing bracket).
                if b == b']' && j > 0 && bytes[j - 1] == b' ' && bytes.get(j.saturating_sub(2)) != Some(&b'[') {
                    results.push(MatchResult::new(
                        self.id(),
                        "Too many spaces inside brackets",
                        file.path.clone(),
                        Location { line: i + 1, column: j + 1 },
                        self.severity(),
                    ));
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
        YamlBracketsRule.check_raw_file(&file)
    }

    #[test]
    fn tight_brackets_ok() { assert!(lint("list: [1, 2, 3]\n").is_empty()); }

    #[test]
    fn empty_brackets_ok() { assert!(lint("list: []\n").is_empty()); }

    #[test]
    fn space_after_open_flagged() {
        let r = lint("list: [ 1, 2]\n");
        assert!(!r.is_empty());
    }
}
