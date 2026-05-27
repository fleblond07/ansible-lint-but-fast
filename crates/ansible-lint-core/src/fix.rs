/// Auto-fix engine: transforms file content to resolve certain rule violations.
///
/// Each fixer receives the file content and returns an (optionally) modified content.
/// Fixers are applied in rule_id order; at most one pass is done per invocation.
use std::path::Path;

use crate::rule::MatchResult;

/// Apply auto-fixes to a file, returning (new_content, count_of_fixes_applied).
/// Only rules listed in `write_list` are fixed (empty list = fix all fixable rules).
pub fn apply_fixes(
    content: &str,
    results: &[MatchResult],
    write_list: &[String],
) -> (String, usize) {
    let mut current = content.to_string();
    let mut count = 0;

    // Collect unique rule IDs present in results that are fixable.
    let rule_ids: Vec<&str> = results.iter()
        .map(|m| m.rule_id.as_str())
        .filter(|id| write_list.is_empty() || write_list.iter().any(|w| w == *id))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for rule_id in &rule_ids {
        let (fixed, n) = fix_rule(&current, rule_id);
        current = fixed;
        count += n;
    }

    (current, count)
}

fn fix_rule(content: &str, rule_id: &str) -> (String, usize) {
    match rule_id {
        "yaml[truthy]" => fix_truthy(content),
        "yaml[trailing-spaces]" => fix_trailing_spaces(content),
        "yaml[new-line-at-end-of-file]" => fix_new_line_at_eof(content),
        "yaml[document-start]" => fix_document_start(content),
        _ => (content.to_string(), 0),
    }
}

/// Replace non-canonical boolean values with true/false.
fn fix_truthy(content: &str) -> (String, usize) {
    let replacements = [
        (": yes\n", ": true\n"),
        (": no\n", ": false\n"),
        (": on\n", ": true\n"),
        (": off\n", ": false\n"),
        (": Yes\n", ": true\n"),
        (": No\n", ": false\n"),
        (": On\n", ": true\n"),
        (": Off\n", ": false\n"),
        (": YES\n", ": true\n"),
        (": NO\n", ": false\n"),
        (": ON\n", ": true\n"),
        (": OFF\n", ": false\n"),
        // Handle end-of-file (no trailing newline).
        (": yes", ": true"),
        (": no", ": false"),
        (": on", ": true"),
        (": off", ": false"),
    ];

    let mut result = content.to_string();
    let mut count = 0;

    for (from, to) in &replacements {
        let new = result.replace(from, to);
        if new != result {
            count += result.matches(from).count();
            result = new;
        }
    }

    (result, count)
}

/// Remove trailing whitespace from each line.
fn fix_trailing_spaces(content: &str) -> (String, usize) {
    let mut count = 0;
    let fixed: Vec<&str> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_end();
            if trimmed.len() != line.len() {
                count += 1;
            }
            trimmed
        })
        .collect();

    // Reconstruct with same line endings.
    let mut result = fixed.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }

    (result, count)
}

/// Ensure file ends with a newline.
fn fix_new_line_at_eof(content: &str) -> (String, usize) {
    if content.is_empty() || content.ends_with('\n') {
        (content.to_string(), 0)
    } else {
        (format!("{content}\n"), 1)
    }
}

/// Add `---` document start marker if missing.
fn fix_document_start(content: &str) -> (String, usize) {
    let first_non_empty = content.lines().find(|l| !l.trim().is_empty());
    if first_non_empty.is_none_or(|l| !l.trim_start().starts_with("---")) {
        (format!("---\n{content}"), 1)
    } else {
        (content.to_string(), 0)
    }
}

/// Apply fixes to a file on disk, return number of fixes made.
pub fn fix_file(path: &Path, results: &[MatchResult], write_list: &[String]) -> std::io::Result<usize> {
    let content = std::fs::read_to_string(path)?;
    let file_results: Vec<&MatchResult> = results.iter()
        .filter(|m| m.filename == path)
        .collect();

    if file_results.is_empty() {
        return Ok(0);
    }

    let (fixed, count) = apply_fixes(&content, &file_results.into_iter().cloned().collect::<Vec<_>>(), write_list);
    if count > 0 {
        std::fs::write(path, fixed)?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_truthy_yes() {
        let (out, count) = fix_truthy("enabled: yes\n");
        assert_eq!(out, "enabled: true\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn fix_trailing_spaces_basic() {
        let (out, count) = fix_trailing_spaces("foo: bar   \nbaz: qux\n");
        assert_eq!(out, "foo: bar\nbaz: qux\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn fix_new_line_adds_newline() {
        let (out, count) = fix_new_line_at_eof("foo: bar");
        assert_eq!(out, "foo: bar\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn fix_new_line_already_present() {
        let (out, count) = fix_new_line_at_eof("foo: bar\n");
        assert_eq!(out, "foo: bar\n");
        assert_eq!(count, 0);
    }

    #[test]
    fn fix_document_start_adds_marker() {
        let (out, count) = fix_document_start("- name: task\n");
        assert_eq!(out, "---\n- name: task\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn fix_document_start_already_present() {
        let (out, count) = fix_document_start("---\n- name: task\n");
        assert_eq!(out, "---\n- name: task\n");
        assert_eq!(count, 0);
    }
}
