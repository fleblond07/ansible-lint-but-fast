use std::path::{Path, PathBuf};

/// Represents a discovered Ansible role directory.
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub root: PathBuf,
    pub tasks_dir: Option<PathBuf>,
    pub handlers_dir: Option<PathBuf>,
    pub vars_dir: Option<PathBuf>,
    pub defaults_dir: Option<PathBuf>,
    pub meta_dir: Option<PathBuf>,
}

impl Role {
    pub fn from_path(root: &Path) -> Option<Self> {
        if !root.is_dir() {
            return None;
        }
        let name = root.file_name()?.to_string_lossy().into_owned();

        let dir = |sub: &str| {
            let p = root.join(sub);
            if p.is_dir() { Some(p) } else { None }
        };

        Some(Role {
            name,
            root: root.to_path_buf(),
            tasks_dir: dir("tasks"),
            handlers_dir: dir("handlers"),
            vars_dir: dir("vars"),
            defaults_dir: dir("defaults"),
            meta_dir: dir("meta"),
        })
    }
}

/// Detect if `path` lives inside a role directory and return the role root.
pub fn role_root_for(path: &Path) -> Option<PathBuf> {
    let role_subdirs = ["tasks", "handlers", "vars", "defaults", "meta", "templates", "files"];
    let mut current = path.parent()?;
    loop {
        let dir_name = current.file_name()?.to_string_lossy();
        if role_subdirs.contains(&dir_name.as_ref()) {
            // Parent of this is potentially the role root.
            let candidate = current.parent()?;
            if candidate.join("tasks").is_dir() || candidate.join("meta").is_dir() {
                return Some(candidate.to_path_buf());
            }
        }
        current = current.parent()?;
    }
}
