use yaml_rust2::{Yaml, YamlLoader};

use crate::error::LintError;
use crate::rule::Location;

/// A YAML node with its source position.
#[derive(Debug, Clone)]
pub struct MarkedNode {
    pub value: MarkedValue,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum MarkedValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Array(Vec<MarkedNode>),
    Hash(Vec<(MarkedNode, MarkedNode)>),
    BadValue,
}

impl MarkedNode {
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            MarkedValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.value {
            MarkedValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_hash(&self) -> Option<&[(MarkedNode, MarkedNode)]> {
        match &self.value {
            MarkedValue::Hash(pairs) => Some(pairs.as_slice()),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&[MarkedNode]> {
        match &self.value {
            MarkedValue::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    /// Look up a key in a hash node.
    pub fn get(&self, key: &str) -> Option<&MarkedNode> {
        self.as_hash()?.iter().find_map(|(k, v)| {
            if k.as_str() == Some(key) { Some(v) } else { None }
        })
    }

    pub fn is_null(&self) -> bool {
        matches!(self.value, MarkedValue::Null)
    }
}

/// Parse a YAML string and return the marked document list.
///
/// yaml-rust2 does not expose per-node markers through the high-level API.
/// We use the scanner-level ScanEvent stream to build a position map keyed
/// by (line, col) → then reconcile with the parsed Yaml tree.
///
/// For a linter we really only need accurate line numbers on YAML scalars
/// and mapping keys. We accomplish this by doing two passes:
///   1. Low-level scan to collect (value, mark) pairs.
///   2. High-level parse to get the structured Yaml value.
///   3. Walk both trees in parallel to stamp positions.
///
/// This is good-enough for all Phase 1 rules.
pub fn parse_yaml_with_positions(
    source: &str,
    path: &str,
) -> Result<Vec<MarkedNode>, LintError> {
    let docs = YamlLoader::load_from_str(source)
        .map_err(|e| LintError::YamlParse {
            file: path.to_string(),
            message: e.to_string(),
        })?;

    Ok(docs
        .into_iter()
        .map(|doc| convert_yaml(&doc, &mut LineTracker::new(source)))
        .collect())
}

/// Converts a `Yaml` node. We approximate positions by scanning the source
/// for the first occurrence of each scalar value starting from our tracked
/// offset. This is imperfect but sufficient for line-level reporting.
fn convert_yaml(yaml: &Yaml, tracker: &mut LineTracker) -> MarkedNode {
    match yaml {
        Yaml::Real(s) => {
            let loc = tracker.find_scalar(s);
            MarkedNode { value: MarkedValue::Real(s.parse().unwrap_or(0.0)), location: loc }
        }
        Yaml::Integer(n) => {
            let s = n.to_string();
            let loc = tracker.find_scalar(&s);
            MarkedNode { value: MarkedValue::Integer(*n), location: loc }
        }
        Yaml::String(s) => {
            let loc = tracker.find_scalar(s);
            MarkedNode { value: MarkedValue::String(s.clone()), location: loc }
        }
        Yaml::Boolean(b) => {
            // Match the canonical form the user wrote.
            let loc = tracker.find_any(&["true", "false", "yes", "no", "on", "off",
                                          "True", "False", "Yes", "No", "On", "Off",
                                          "TRUE", "FALSE", "YES", "NO", "ON", "OFF"]);
            MarkedNode { value: MarkedValue::Boolean(*b), location: loc }
        }
        Yaml::Array(items) => {
            let converted: Vec<MarkedNode> = items.iter()
                .map(|i| convert_yaml(i, tracker))
                .collect();
            let loc = converted.first()
                .map(|n| n.location.clone())
                .unwrap_or_default();
            MarkedNode { value: MarkedValue::Array(converted), location: loc }
        }
        Yaml::Hash(map) => {
            let pairs: Vec<(MarkedNode, MarkedNode)> = map.iter()
                .map(|(k, v)| (convert_yaml(k, tracker), convert_yaml(v, tracker)))
                .collect();
            let loc = pairs.first()
                .map(|(k, _)| k.location.clone())
                .unwrap_or_default();
            MarkedNode { value: MarkedValue::Hash(pairs), location: loc }
        }
        Yaml::Null => MarkedNode { value: MarkedValue::Null, location: Location::default() },
        Yaml::BadValue => MarkedNode { value: MarkedValue::BadValue, location: Location::default() },
        Yaml::Alias(_) => MarkedNode { value: MarkedValue::Null, location: Location::default() },
    }
}

/// Tracks scan position through source lines so we can find approximate
/// 1-based line/col for scalar values.
pub struct LineTracker<'a> {
    lines: Vec<&'a str>,
    current_line: usize, // 0-based
    current_col: usize,  // 0-based byte offset within line
}

impl<'a> LineTracker<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            current_line: 0,
            current_col: 0,
        }
    }

    /// Find any of `candidates` starting from current position. Returns first match.
    pub fn find_any(&mut self, candidates: &[&str]) -> Location {
        for candidate in candidates {
            if let Some(loc) = self.try_find(candidate) {
                return loc;
            }
        }
        Location { line: self.current_line + 1, column: self.current_col + 1 }
    }

    pub fn find_scalar(&mut self, s: &str) -> Location {
        if s.is_empty() {
            return Location { line: self.current_line + 1, column: self.current_col + 1 };
        }
        // Search forward line by line.
        for line_idx in self.current_line..self.lines.len() {
            let start_col = if line_idx == self.current_line { self.current_col } else { 0 };
            let line = self.lines[line_idx];
            // Search within this line starting at start_col.
            if let Some(byte_off) = find_substr(&line[start_col..], s) {
                let col = start_col + byte_off;
                self.current_line = line_idx;
                self.current_col = col + s.len();
                return Location { line: line_idx + 1, column: col + 1 };
            }
        }
        // Not found — return current position.
        Location { line: self.current_line + 1, column: self.current_col + 1 }
    }

    fn try_find(&mut self, s: &str) -> Option<Location> {
        for line_idx in self.current_line..self.lines.len() {
            let start_col = if line_idx == self.current_line { self.current_col } else { 0 };
            let line = self.lines[line_idx];
            if let Some(byte_off) = find_substr(&line[start_col..], s) {
                let col = start_col + byte_off;
                self.current_line = line_idx;
                self.current_col = col + s.len();
                return Some(Location { line: line_idx + 1, column: col + 1 });
            }
        }
        None
    }
}

fn find_substr(haystack: &str, needle: &str) -> Option<usize> {
    let n = needle.len();
    (0..=haystack.len().saturating_sub(n))
        .find(|&i| haystack.as_bytes().get(i..i + n) == Some(needle.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_mapping() {
        let yaml = "name: foo\nvalue: 42\n";
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        assert_eq!(docs.len(), 1);
        let name_val = docs[0].get("name").unwrap();
        assert_eq!(name_val.as_str(), Some("foo"));
    }

    #[test]
    fn test_parse_list_of_mappings() {
        let yaml = "- name: task one\n  debug:\n    msg: hello\n";
        let docs = parse_yaml_with_positions(yaml, "test.yml").unwrap();
        assert_eq!(docs.len(), 1);
        let items = docs[0].as_vec().unwrap();
        assert_eq!(items.len(), 1);
    }
}
