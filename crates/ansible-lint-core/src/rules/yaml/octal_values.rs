use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Octal values must use the `0o` prefix, not legacy `0` prefix.
/// Rule ID: yaml[octal-values]
pub struct YamlOctalValuesRule;

impl Rule for YamlOctalValuesRule {
    fn id(&self) -> &str { "yaml[octal-values]" }
    fn description(&self) -> &str { "Octal values must use 0o prefix (e.g. 0o755 not 0755)" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Error }
    fn tags(&self) -> &[&str] { &["yaml"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Safety] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for (i, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            // Look for bare octal literals: 0[0-7]+ not followed by x, o, b, or digits 8-9.
            // Pattern: value starts with 0 followed only by 0-7 and has length >= 2.
            if let Some(colon) = trimmed.find(':') {
                let value_part = trimmed[colon + 1..].trim();
                // Strip trailing comment.
                let value = value_part.split('#').next().unwrap_or("").trim();
                // Check for quoted octal (which is intentional as string).
                if value.starts_with('"') || value.starts_with('\'') {
                    continue;
                }
                if is_legacy_octal(value) {
                    let col = line.find(value).unwrap_or(0) + 1;
                    results.push(MatchResult::new(
                        self.id(),
                        format!("Use 0o prefix for octal values: '0o{}'  instead of '{value}'", &value[1..]),
                        file.path.clone(),
                        Location { line: i + 1, column: col },
                        self.severity(),
                    ));
                }
            }
        }

        results
    }
}

fn is_legacy_octal(s: &str) -> bool {
    if s.len() < 2 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[0] != b'0' {
        return false;
    }
    // Must not be 0x (hex), 0o (new octal), 0b (binary), or just 0.
    if matches!(bytes[1], b'x' | b'X' | b'o' | b'O' | b'b' | b'B') {
        return false;
    }
    // All remaining chars must be octal digits.
    bytes[1..].iter().all(|&b| matches!(b, b'0'..=b'7'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::FileKind;
    use std::path::PathBuf;

    fn lint(content: &str) -> Vec<MatchResult> {
        let file = LintFile { path: PathBuf::from("t.yml"), content: content.to_string(), kind: FileKind::Tasks };
        YamlOctalValuesRule.check_raw_file(&file)
    }

    #[test]
    fn new_octal_ok() { assert!(lint("mode: 0o755\n").is_empty()); }

    #[test]
    fn quoted_ok() { assert!(lint("mode: '0755'\n").is_empty()); }

    #[test]
    fn legacy_octal_flagged() {
        let r = lint("mode: 0755\n");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn zero_ok() { assert!(lint("count: 0\n").is_empty()); }
}
