use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Comments must be preceded by a space (except for shebangs and YAML directives).
/// Rule ID: yaml[comments]
pub struct YamlCommentsRule;

impl Rule for YamlCommentsRule {
    fn id(&self) -> &str { "yaml[comments]" }
    fn description(&self) -> &str { "Comment must start with a space after '#'" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (i, line) in file.content.lines().enumerate() {
            let trimmed = line.trim_start();

            // Shebang on line 1
            if i == 0 && trimmed.starts_with("#!") {
                continue;
            }
            // YAML directive (e.g. %YAML 1.2)
            if trimmed.starts_with('%') {
                continue;
            }

            // Find inline or standalone comment markers.
            // We scan for '#' that's not inside a string (simplified: skip quoted regions).
            let bytes = line.as_bytes();
            let mut in_single = false;
            let mut in_double = false;

            for (j, &b) in bytes.iter().enumerate() {
                match b {
                    b'\'' if !in_double => in_single = !in_single,
                    b'"' if !in_single => in_double = !in_double,
                    b'#' if !in_single && !in_double => {
                        // Is there a non-space character immediately after #?
                        let after = bytes.get(j + 1).copied();
                        if let Some(next) = after {
                            if next != b' ' && next != b'\n' && next != b'!' {
                                // Also skip `#noqa` annotation markers.
                                let rest = &line[j + 1..];
                                if rest.starts_with("noqa") {
                                    break;
                                }
                                results.push(MatchResult::new(
                                    self.id(),
                                    "Comment must start with a space after '#'",
                                    file.path.clone(),
                                    Location { line: i + 1, column: j + 1 },
                                    self.severity(),
                                ));
                            }
                        }
                        break; // Only first # per line
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
        YamlCommentsRule.check_raw_file(&file)
    }

    #[test]
    fn good_comment_ok() { assert!(lint("# This is fine\nfoo: bar\n").is_empty()); }

    #[test]
    fn inline_good_comment_ok() { assert!(lint("foo: bar # inline comment\n").is_empty()); }

    #[test]
    fn bad_comment_flagged() {
        let r = lint("#bad comment\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn shebang_ok() { assert!(lint("#!/usr/bin/env ansible-playbook\n").is_empty()); }
}
