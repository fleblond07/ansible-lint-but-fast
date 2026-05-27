use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::error::LintError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileKind {
    Playbook,
    Tasks,
    Handlers,
    Vars,
    Defaults,
    Meta,
    Requirements,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub kind: FileKind,
}

/// Discover all Ansible YAML files under `roots`, applying exclusion globs.
pub fn discover_files(
    roots: &[PathBuf],
    exclude_globs: &[String],
) -> Result<Vec<DiscoveredFile>, LintError> {
    let excludes = build_globset(exclude_globs)?;
    let mut files = Vec::new();

    for root in roots {
        let walker = WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden subdirs (like .git) but not the root itself.
                if e.depth() > 0 {
                    let name = e.file_name().to_string_lossy();
                    if name.starts_with('.') && e.file_type().is_dir() {
                        return false;
                    }
                }
                true
            });

        for entry in walker {
            let entry = entry.map_err(|e| LintError::Io(e.into()))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yml" && ext != "yaml" {
                continue;
            }

            // Check exclusion globs.
            let path_str = path.to_string_lossy();
            if excludes.is_match(path_str.as_ref()) {
                continue;
            }

            let kind = classify(path);
            files.push(DiscoveredFile {
                path: path.to_path_buf(),
                kind,
            });
        }
    }

    Ok(files)
}

fn classify(path: &Path) -> FileKind {
    // Check filename first.
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "requirements.yml" || file_name == "requirements.yaml" {
        return FileKind::Requirements;
    }

    // Walk parent dirs to find role structure indicators.
    let parts: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    for part in parts.iter() {
        match *part {
            "tasks" => return FileKind::Tasks,
            "handlers" => return FileKind::Handlers,
            "vars" => return FileKind::Vars,
            "defaults" => return FileKind::Defaults,
            "meta" => return FileKind::Meta,
            _ => {}
        }
    }

    // No role subdir found — likely a playbook or unknown.
    // We'll classify playbooks vs tasks during parsing (needs YAML content).
    FileKind::Unknown
}

fn build_globset(globs: &[String]) -> Result<GlobSet, LintError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in globs {
        let glob = Glob::new(pattern)
            .map_err(|e| LintError::Config(format!("Invalid glob '{pattern}': {e}")))?;
        builder.add(glob);
    }
    builder.build().map_err(|e| LintError::Config(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_yaml_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("site.yml"), "---\n").unwrap();
        fs::create_dir(dir.path().join("tasks")).unwrap();
        fs::write(dir.path().join("tasks/main.yml"), "---\n").unwrap();
        fs::write(dir.path().join("readme.txt"), "hello").unwrap();

        let files = discover_files(&[dir.path().to_path_buf()], &[]).unwrap();
        assert_eq!(files.len(), 2);
        let kinds: Vec<_> = files.iter().map(|f| &f.kind).collect();
        assert!(kinds.contains(&&FileKind::Tasks));
    }

    #[test]
    fn test_exclude_glob() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.yml"), "---\n").unwrap();
        fs::write(dir.path().join("skip.yml"), "---\n").unwrap();

        let files = discover_files(
            &[dir.path().to_path_buf()],
            &["**/skip.yml".to_string()],
        ).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].path.file_name().unwrap() == "main.yml");
    }
}
