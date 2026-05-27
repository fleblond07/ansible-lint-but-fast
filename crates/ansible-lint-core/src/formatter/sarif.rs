use serde::Serialize;
use serde_json;

use crate::formatter::Formatter;
use crate::rule::MatchResult;

pub struct SarifFormatter;

#[derive(Serialize)]
struct SarifLog<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun<'a>>,
}

#[derive(Serialize)]
struct SarifRun<'a> {
    tool: SarifTool,
    results: Vec<SarifResult<'a>>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
}

#[derive(Serialize)]
struct SarifResult<'a> {
    #[serde(rename = "ruleId")]
    rule_id: &'a str,
    message: SarifMessage<'a>,
    locations: Vec<SarifLocation<'a>>,
    level: &'static str,
}

#[derive(Serialize)]
struct SarifMessage<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct SarifLocation<'a> {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation<'a>,
}

#[derive(Serialize)]
struct SarifPhysicalLocation<'a> {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation<'a>,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation<'a> {
    uri: &'a str,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
}

impl Formatter for SarifFormatter {
    fn format(&self, results: &[MatchResult], _color: bool) -> String {
        use crate::rule::Severity;

        let sarif_results: Vec<SarifResult> = results.iter().map(|m| {
            let level = match m.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };
            SarifResult {
                rule_id: &m.rule_id,
                message: SarifMessage { text: &m.message },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: m.filename.to_str().unwrap_or(""),
                        },
                        region: SarifRegion {
                            start_line: m.location.line,
                            start_column: m.location.column,
                        },
                    },
                }],
                level,
            }
        }).collect();

        let log = SarifLog {
            schema: "https://json.schemastore.org/sarif-2.1.0.json",
            version: "2.1.0",
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "ansible-lint",
                        version: env!("CARGO_PKG_VERSION"),
                        information_uri: "https://github.com/ansible/ansible-lint",
                    },
                },
                results: sarif_results,
            }],
        };

        serde_json::to_string_pretty(&log).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Location, Severity};
    use std::path::PathBuf;

    #[test]
    fn sarif_output_valid() {
        let r = MatchResult::new(
            "yaml[truthy]", "Use true/false",
            PathBuf::from("test.yml"),
            Location { line: 3, column: 1 },
            Severity::Warning,
        );
        let fmt = SarifFormatter;
        let out = fmt.format(&[r], false);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["version"], "2.1.0");
        assert!(v["runs"][0]["results"][0]["ruleId"].is_string());
    }
}
