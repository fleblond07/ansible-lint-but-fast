use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Colons in YAML must be followed by a space (or be at end of line).
/// Rule ID: yaml[colons]
pub struct YamlColonsRule;

impl Rule for YamlColonsRule {
    fn id(&self) -> &str { "yaml[colons]" }
    fn description(&self) -> &str { "Colons must be followed by a space" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (i, line) in file.content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || trimmed == "---" { continue; }

            let bytes = line.as_bytes();
            let mut in_single = false;
            let mut in_double = false;

            for (j, &b) in bytes.iter().enumerate() {
                match b {
                    b'\'' if !in_double => in_single = !in_single,
                    b'"' if !in_single => in_double = !in_double,
                    b':' if !in_single && !in_double => {
                        let next = bytes.get(j + 1).copied();
                        match next {
                            // OK: end of line, followed by space, or followed by newline.
                            None | Some(b' ') | Some(b'\n') | Some(b'\r') => {}
                            // OK: `://` pattern (URLs).
                            Some(b'/') if bytes.get(j + 2) == Some(&b'/') => {}
                            _ => {
                                results.push(MatchResult::new(
                                    self.id(),
                                    "Colon must be followed by a space",
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
        YamlColonsRule.check_raw_file(&file)
    }

    #[test]
    fn normal_colon_ok() { assert!(lint("foo: bar\n").is_empty()); }

    #[test]
    fn url_ok() { assert!(lint("url: https://example.com\n").is_empty()); }

    #[test]
    fn no_space_after_colon_flagged() {
        let r = lint("foo:bar\n");
        assert!(!r.is_empty());
    }
}
