use std::path::PathBuf;

use crate::parser::task::Task;
use crate::parser::playbook::Play;
use crate::registry::Profile;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Source location within a file (1-based).
#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

/// A single rule violation.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub rule_id: String,
    pub message: String,
    pub filename: PathBuf,
    pub location: Location,
    pub severity: Severity,
    /// Sub-tag (e.g. "yaml[truthy]" is both rule_id and tag here).
    pub tag: Option<String>,
    pub task_name: Option<String>,
}

impl MatchResult {
    pub fn new(
        rule_id: impl Into<String>,
        message: impl Into<String>,
        filename: impl Into<PathBuf>,
        location: Location,
        severity: Severity,
    ) -> Self {
        let rule_id = rule_id.into();
        let tag = Some(rule_id.clone());
        Self {
            rule_id,
            message: message.into(),
            filename: filename.into(),
            location,
            severity,
            tag,
            task_name: None,
        }
    }

    pub fn with_task_name(mut self, name: impl Into<String>) -> Self {
        self.task_name = Some(name.into());
        self
    }
}

/// A lint file representation passed to rules.
pub struct LintFile {
    pub path: PathBuf,
    /// Raw file contents (for text-level checks).
    pub content: String,
    pub kind: crate::discovery::FileKind,
}

/// The core trait every rule implements.
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn help_url(&self) -> &str;
    fn severity(&self) -> Severity;
    fn tags(&self) -> &[&str];
    fn profiles(&self) -> &[Profile];

    /// Called once per file. Useful for raw-text and YAML structure checks.
    fn check_raw_file(&self, _file: &LintFile) -> Vec<MatchResult> {
        vec![]
    }

    /// Called once per play in a playbook.
    fn check_play(&self, _play: &Play, _file: &LintFile) -> Vec<MatchResult> {
        vec![]
    }

    /// Called once per task (includes handlers).
    fn check_task(&self, _task: &Task, _file: &LintFile) -> Vec<MatchResult> {
        vec![]
    }
}
