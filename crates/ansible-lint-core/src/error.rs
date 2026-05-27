use thiserror::Error;

#[derive(Debug, Error)]
pub enum LintError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error in {file}: {message}")]
    YamlParse { file: String, message: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Invalid rule ID: {0}")]
    InvalidRule(String),
}
