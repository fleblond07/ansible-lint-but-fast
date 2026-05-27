use crate::rule::Rule;

/// Ordered severity — higher index = stricter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Min,
    #[default]
    Basic,
    Moderate,
    Safety,
    Shared,
    Production,
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Profile::Min => "min",
            Profile::Basic => "basic",
            Profile::Moderate => "moderate",
            Profile::Safety => "safety",
            Profile::Shared => "shared",
            Profile::Production => "production",
        };
        write!(f, "{s}")
    }
}

impl Profile {
    pub fn description(&self) -> &'static str {
        match self {
            Profile::Min => "Mandatory rules that prevent fatal errors",
            Profile::Basic => "Common coding issues and style enforcement",
            Profile::Moderate => "Best practices for maintainability",
            Profile::Safety => "Avoids non-deterministic or security-risky calls",
            Profile::Shared => "Additional strictness for shared collections",
            Profile::Production => "Maximum strictness for production workloads",
        }
    }

    /// Returns all profiles at or below this profile's strictness level.
    pub fn includes(&self) -> &'static [Profile] {
        match self {
            Profile::Min => &[Profile::Min],
            Profile::Basic => &[Profile::Min, Profile::Basic],
            Profile::Moderate => &[Profile::Min, Profile::Basic, Profile::Moderate],
            Profile::Safety => &[Profile::Min, Profile::Basic, Profile::Moderate, Profile::Safety],
            Profile::Shared => &[Profile::Min, Profile::Basic, Profile::Moderate, Profile::Safety, Profile::Shared],
            Profile::Production => &[Profile::Min, Profile::Basic, Profile::Moderate, Profile::Safety, Profile::Shared, Profile::Production],
        }
    }
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    /// Return rules active under the given profile, minus skip_list, plus enable_list.
    pub fn active_rules(
        &self,
        profile: Profile,
        skip_list: &[String],
        enable_list: &[String],
    ) -> Vec<&dyn Rule> {
        let active_profiles = profile.includes();
        self.rules
            .iter()
            .filter(|r| {
                let in_profile = r.profiles().iter().any(|p| active_profiles.contains(p));
                let explicitly_enabled = enable_list.iter().any(|e| e == r.id() || r.tags().contains(&e.as_str()));
                let skipped = skip_list.iter().any(|s| s == r.id() || r.tags().contains(&s.as_str()));
                (in_profile || explicitly_enabled) && !skipped
            })
            .map(|r| r.as_ref())
            .collect()
    }

    pub fn all_rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    pub fn all_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self.rules.iter()
            .flat_map(|r| r.tags().iter().copied())
            .collect();
        tags.sort_unstable();
        tags.dedup();
        tags
    }
}
