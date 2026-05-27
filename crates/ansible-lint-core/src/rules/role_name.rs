use crate::registry::Profile;
use crate::rule::{LintFile, Location, MatchResult, Rule, Severity};
use crate::parser::role::role_root_for;

/// Role names must use snake_case and not start with digits or contain special chars.
/// Rule ID: role-name
pub struct RoleNameRule;

impl Rule for RoleNameRule {
    fn id(&self) -> &str { "role-name" }
    fn description(&self) -> &str { "Role names must use snake_case" }
    fn help_url(&self) -> &str { "https://ansible.readthedocs.io/projects/lint/rules/role-name/" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn tags(&self) -> &[&str] { &["idiom"] }
    fn profiles(&self) -> &[Profile] { &[Profile::Basic] }

    fn check_raw_file(&self, file: &LintFile) -> Vec<MatchResult> {
        // Only check files inside roles.
        let role_root = match role_root_for(&file.path) {
            Some(r) => r,
            None => return vec![],
        };

        let role_name = match role_root.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => return vec![],
        };

        if !is_valid_role_name(&role_name) {
            vec![MatchResult::new(
                self.id(),
                format!("Role name '{role_name}' is not valid; use snake_case (a-z, 0-9, _) and must start with a letter"),
                file.path.clone(),
                Location { line: 1, column: 1 },
                self.severity(),
            )]
        } else {
            vec![]
        }
    }
}

fn is_valid_role_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return false;
    }
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::is_valid_role_name;

    #[test]
    fn valid_names() {
        assert!(is_valid_role_name("my_role"));
        assert!(is_valid_role_name("nginx"));
        assert!(is_valid_role_name("apache2"));
    }

    #[test]
    fn invalid_names() {
        assert!(!is_valid_role_name("MyRole"));
        assert!(!is_valid_role_name("my-role"));
        assert!(!is_valid_role_name("1bad"));
        assert!(!is_valid_role_name(""));
    }
}
