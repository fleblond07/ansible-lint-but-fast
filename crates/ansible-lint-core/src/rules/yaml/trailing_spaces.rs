use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Lines must not have trailing whitespace.
/// Rule ID: yaml[trailing-spaces]
pub struct YamlTrailingSpacesRule;

impl Rule for YamlTrailingSpacesRule {
    fn id(&self) -> &str { "yaml[trailing-spaces]" }
    fn description(&self) -> &str { "Lines must not have trailing whitespace" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/yaml/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["yaml", "formatting"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        file.content
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                if line != line.trim_end() {
                    let trailing_start = line.trim_end().len() + 1;
                    Some(MatchResult::new(
                        self.id(),
                        "Trailing spaces found",
                        file.path.clone(),
                        Location { line: i + 1, column: trailing_start },
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
        YamlTrailingSpacesRule.check_raw_file(&file)
    }

    #[test]
    fn no_trailing_spaces_ok() { assert!(lint("name: foo\n").is_empty()); }

    #[test]
    fn trailing_space_flagged() {
        let r = lint("name: foo   \n");
        assert_eq!(r.len(), 1);
    }
}
