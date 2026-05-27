use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::LintError;
use crate::registry::Profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub profile: Profile,
    pub skip_list: Vec<String>,
    pub warn_list: Vec<String>,
    pub enable_list: Vec<String>,
    pub exclude_paths: Vec<String>,
    /// Regex pattern variable names must match.
    pub var_naming_pattern: String,
    pub loop_var_prefix: Option<String>,
    pub offline: bool,
    pub strict: bool,
    pub write_list: Vec<String>,
    /// Module names to treat as mock (skip fqcn/only-builtins checks for these).
    pub mock_modules: Vec<String>,
    /// Role names to treat as mock (skip import errors for these).
    pub mock_roles: Vec<String>,
    /// Maximum nesting depth of block/rescue/always structures.
    pub block_depth_limit: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: Profile::Basic,
            skip_list: Vec::new(),
            warn_list: Vec::new(),
            enable_list: Vec::new(),
            exclude_paths: Vec::new(),
            var_naming_pattern: r"^[a-z_][a-z0-9_]*$".to_string(),
            loop_var_prefix: None,
            offline: false,
            strict: false,
            write_list: Vec::new(),
            mock_modules: Vec::new(),
            mock_roles: Vec::new(),
            block_depth_limit: None,
        }
    }
}

impl Config {
    /// Search upward from `start` for a config file and load it.
    /// Falls back to `Config::default()` if none found.
    pub fn load(start: &Path) -> Result<(Self, Option<PathBuf>), LintError> {
        let candidates = [
            ".ansible-lint",
            ".ansible-lint.yml",
            ".ansible-lint.yaml",
            ".config/ansible-lint.yml",
            ".config/ansible-lint.yaml",
        ];

        let mut dir = start.to_path_buf();
        loop {
            for name in &candidates {
                let candidate = dir.join(name);
                if candidate.exists() {
                    let cfg = Self::from_file(&candidate)?;
                    return Ok((cfg, Some(candidate)));
                }
            }
            if !dir.pop() {
                break;
            }
        }

        Ok((Config::default(), None))
    }

    /// Load config from an explicit file path.
    pub fn from_file(path: &Path) -> Result<Self, LintError> {
        let content = std::fs::read_to_string(path)
            .map_err(LintError::Io)?;

        // Try YAML first, then TOML.
        if path.extension().is_some_and(|e| e == "toml") {
            toml::from_str::<Config>(&content)
                .map_err(|e| LintError::Config(e.to_string()))
        } else {
            serde_yaml::from_str::<Config>(&content)
                .map_err(|e| LintError::Config(e.to_string()))
        }
    }

    /// Merge CLI overrides into the config. CLI values take precedence.
    #[allow(clippy::too_many_arguments)]
    pub fn merge_cli(
        &mut self,
        profile: Option<Profile>,
        skip_list: Vec<String>,
        warn_list: Vec<String>,
        enable_list: Vec<String>,
        exclude_paths: Vec<String>,
        offline: bool,
        strict: bool,
    ) {
        if let Some(p) = profile {
            self.profile = p;
        }
        // CLI lists are appended (matching upstream behaviour).
        self.skip_list.extend(skip_list);
        self.warn_list.extend(warn_list);
        self.enable_list.extend(enable_list);
        self.exclude_paths.extend(exclude_paths);
        if offline {
            self.offline = true;
        }
        if strict {
            self.strict = true;
        }
    }
}
