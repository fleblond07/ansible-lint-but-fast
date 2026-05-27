use crate::formatter::Formatter;
use crate::formatter::brief::format_line;
use crate::rule::MatchResult;

const CONTEXT_LINES: usize = 2;

/// Like `brief` but includes surrounding source context lines.
pub struct FullFormatter;

impl Formatter for FullFormatter {
    fn format(&self, results: &[MatchResult], color: bool) -> String {
        let mut out = Vec::new();

        for m in results {
            out.push(format_line(m, color));

            // Try to read the file and show context.
            if let Ok(content) = std::fs::read_to_string(&m.filename) {
                let lines: Vec<&str> = content.lines().collect();
                let line_idx = m.location.line.saturating_sub(1); // 0-based
                let start = line_idx.saturating_sub(CONTEXT_LINES);
                let end = (line_idx + CONTEXT_LINES + 1).min(lines.len());

                out.push(String::new());
                for (i, &line) in lines[start..end].iter().enumerate() {
                    let num = start + i + 1;
                    let marker = if num == m.location.line { ">" } else { " " };
                    out.push(format!("{marker} {num:>4} | {line}"));
                }
                out.push(String::new());
            }
        }

        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Location, Severity};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_full_format_with_context() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "line1\nline2\nline3: yes\nline4\nline5").unwrap();
        let path = f.path().to_path_buf();

        let r = MatchResult::new(
            "yaml[truthy]",
            "Use true/false",
            path.clone(),
            Location { line: 3, column: 8 },
            Severity::Error,
        );

        let fmt = FullFormatter;
        let out = fmt.format(&[r], false);
        assert!(out.contains(">    3 | line3: yes"));
        assert!(out.contains("     2 | line2"));
    }
}
