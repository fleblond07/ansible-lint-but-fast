use crate::discovery::FileKind;
use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};

/// Role meta/main.yml must have author, description, and license.
/// Rule ID: meta-no-info
pub struct MetaNoInfoRule;

impl Rule for MetaNoInfoRule {
    fn id(&self) -> &str { "meta-no-info" }
    fn description(&self) -> &str { "Role meta/main.yml is missing required fields" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/meta-no-info/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["metadata"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        // Only apply to meta files.
        if file.kind != FileKind::Meta {
            return vec![];
        }

        let mut results = Vec::new();
        let content = &file.content;

        // Check for required fields in galaxy_info section.
        let required = ["author", "description", "license"];
        for field in &required {
            if !content.contains(&format!("{field}:")) {
                results.push(MatchResult::new(
                    self.id(),
                    format!("meta/main.yml is missing required galaxy_info field: '{field}'"),
                    file.path.clone(),
                    Location { line: 1, column: 1 },
                    self.severity(),
                ));
            }
        }

        results
    }
}

/// Role meta/main.yml should have min_ansible_version set.
/// Rule ID: meta-incorrect
pub struct MetaIncorrectRule;

impl Rule for MetaIncorrectRule {
    fn id(&self) -> &str { "meta-incorrect" }
    fn description(&self) -> &str { "Role meta/main.yml should specify min_ansible_version" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/meta-incorrect/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["metadata"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        if file.kind != FileKind::Meta {
            return vec![];
        }

        if !file.content.contains("min_ansible_version") {
            vec![MatchResult::new(
                self.id(),
                "meta/main.yml should specify 'min_ansible_version' in galaxy_info",
                file.path.clone(),
                Location { line: 1, column: 1 },
                self.severity(),
            )]
        } else {
            vec![]
        }
    }
}

/// Role meta/main.yml should have categories/tags.
/// Rule ID: meta-no-tags
pub struct MetaNoTagsRule;

impl Rule for MetaNoTagsRule {
    fn id(&self) -> &str { "meta-no-tags" }
    fn description(&self) -> &str { "Role meta/main.yml is missing galaxy_info.galaxy_tags" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/meta-no-tags/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["metadata"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Shared] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        if file.kind != FileKind::Meta {
            return vec![];
        }

        if !file.content.contains("galaxy_tags") && !file.content.contains("categories") {
            vec![MatchResult::new(
                self.id(),
                "meta/main.yml should specify galaxy_tags for discoverability",
                file.path.clone(),
                Location { line: 1, column: 1 },
                self.severity(),
            )]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn meta_file(content: &str) -> LintFile {
        LintFile { path: PathBuf::from("meta/main.yml"), content: content.to_string(), kind: FileKind::Meta }
    }

    #[test]
    fn complete_meta_ok() {
        let f = meta_file("galaxy_info:\n  author: me\n  description: test\n  license: MIT\n  min_ansible_version: '2.9'\n  galaxy_tags: []\n");
        assert!(MetaNoInfoRule.check_raw_file(&f).is_empty());
        assert!(MetaIncorrectRule.check_raw_file(&f).is_empty());
        assert!(MetaNoTagsRule.check_raw_file(&f).is_empty());
    }

    #[test]
    fn missing_author_flagged() {
        let f = meta_file("galaxy_info:\n  description: test\n  license: MIT\n");
        let r = MetaNoInfoRule.check_raw_file(&f);
        assert!(r.iter().any(|m| m.message.contains("author")));
    }

    #[test]
    fn tasks_file_not_checked() {
        let f = LintFile { path: PathBuf::from("tasks/main.yml"), content: "foo: bar\n".to_string(), kind: FileKind::Tasks };
        assert!(MetaNoInfoRule.check_raw_file(&f).is_empty());
    }
}
