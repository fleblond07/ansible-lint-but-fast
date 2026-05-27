use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Commas in flow sequences/mappings must be followed by a space.
/// Rule ID: yaml[commas]
pub struct YamlCommasRule;

impl Rule for YamlCommasRule {
    fn id(&self) -> &str { "yaml[commas]" }
    fn description(&self) -> &str { "Commas must be followed by a space" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (i, line) in file.content.lines().enumerate() {
            if line.trim().starts_with('#') { continue; }

            let bytes = line.as_bytes();
            let mut in_single = false;
            let mut in_double = false;

            for (j, &b) in bytes.iter().enumerate() {
                match b {
                    b'\'' if !in_double => in_single = !in_single,
                    b'"' if !in_single => in_double = !in_double,
                    b',' if !in_single && !in_double => {
                        let next = bytes.get(j + 1).copied();
                        match next {
                            None | Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b']') | Some(b'}') => {}
                            _ => {
                                results.push(MatchResult::new(
                                    self.id(),
                                    "Comma must be followed by a space",
                                    file.path.clone(),
                                    Location { line: i + 1, column: j + 1 },
                                    self.severity(),
                                ));
                            }
                        }
                    }
                    _ => {}
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
        YamlCommasRule.check_raw_file(&file)
    }

    #[test]
    fn spaced_commas_ok() { assert!(lint("list: [1, 2, 3]\n").is_empty()); }

    #[test]
    fn unspaced_comma_flagged() {
        let r = lint("list: [1,2,3]\n");
        assert!(!r.is_empty());
    }
}
