pub mod brief;
pub mod codeclimate;
pub mod full;
pub mod json;
pub mod pep8;
pub mod sarif;

use crate::rule::MatchResult;

pub trait Formatter {
    fn format(&self, results: &[MatchResult], color: bool) -> String;
}

pub fn get_formatter(name: &str) -> Box<dyn Formatter> {
    match name {
        "full" => Box::new(full::FullFormatter),
        "json" => Box::new(json::JsonFormatter),
        "sarif" => Box::new(sarif::SarifFormatter),
        "codeclimate" => Box::new(codeclimate::CodeClimateFormatter),
        "pep8" => Box::new(pep8::Pep8Formatter),
        "md" | "markdown" => Box::new(pep8::MarkdownFormatter),
        "quiet" => Box::new(brief::QuietFormatter),
        _ => Box::new(brief::BriefFormatter), // "brief" is default
    }
}
