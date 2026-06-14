use std::collections::HashSet;
use std::path::Path;

/// A single visible entry in the filesystem explorer tree.
#[derive(Debug, Clone)]
pub struct FsEntry {
    /// Indentation depth (0 = repo root level).
    pub depth: usize,
    /// Display name (just the file or directory name).
    pub name: String,
    /// Path relative to the repo root, using '/' separators.
    pub path: String,
    pub is_dir: bool,
}

/// State for the filesystem file explorer shown in the Files panel.
///
/// Unlike the git-status file tree (`show_file_tree`), this lists *all* files
/// and directories in the working tree — like a regular file browser — so the
/// user can browse and preview any file, not just changed ones. Directories
/// are expanded lazily so large repositories stay responsive.
#[derive(Debug, Default)]
pub struct FileExplorerState {
    /// Whether the explorer is currently shown in place of the git file list.
    pub active: bool,
    /// Flattened list of currently-visible entries (depth-first).
    pub entries: Vec<FsEntry>,
    /// Relative paths of directories that are expanded (children visible).
    pub expanded_dirs: HashSet<String>,
}

/// Safety cap so a pathological tree can't blow up memory / render time.
const MAX_ENTRIES: usize = 20_000;

impl FileExplorerState {
    /// Toggle a directory's expanded/collapsed state.
    pub fn toggle_dir(&mut self, path: &str) {
        if !self.expanded_dirs.remove(path) {
            self.expanded_dirs.insert(path.to_string());
        }
    }

    /// Rebuild `entries` by walking the working tree, descending only into
    /// expanded directories. The `.git` directory is always skipped.
    pub fn rebuild(&mut self, repo_root: &Path) {
        self.entries.clear();
        self.walk(repo_root, "", 0);
    }

    fn walk(&mut self, dir: &Path, rel_prefix: &str, depth: usize) {
        if self.entries.len() >= MAX_ENTRIES {
            return;
        }
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            return;
        };

        let mut items: Vec<(String, bool)> = Vec::new();
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Hide the git metadata directory at the repo root.
            if depth == 0 && name == ".git" {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            items.push((name, is_dir));
        }

        // Directories first, then files; case-insensitive alphabetical within
        // each group (matches the feel of most file browsers).
        items.sort_by(|a, b| match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        });

        for (name, is_dir) in items {
            if self.entries.len() >= MAX_ENTRIES {
                return;
            }
            let rel_path = if rel_prefix.is_empty() {
                name.clone()
            } else {
                format!("{rel_prefix}/{name}")
            };
            let child_abs = dir.join(&name);
            let expand = is_dir && self.expanded_dirs.contains(&rel_path);
            self.entries.push(FsEntry {
                depth,
                name,
                path: rel_path.clone(),
                is_dir,
            });
            if expand {
                self.walk(&child_abs, &rel_path, depth + 1);
            }
        }
    }
}

/// Read a file's text for display in the diff/preview panel.
///
/// Returns `None` for binary files (so we don't render garbage) and truncates
/// very large files. Lossily decodes invalid UTF-8 rather than failing.
pub fn read_file_for_view(abs_path: &Path) -> Option<String> {
    const MAX_BYTES: usize = 2 * 1024 * 1024; // 2 MiB preview cap

    let bytes = std::fs::read(abs_path).ok()?;
    let truncated = bytes.len() > MAX_BYTES;
    let slice = if truncated {
        &bytes[..MAX_BYTES]
    } else {
        &bytes[..]
    };

    // Heuristic: a NUL byte means binary — don't try to preview it.
    if slice.contains(&0) {
        return None;
    }

    let mut text = String::from_utf8_lossy(slice).into_owned();
    if truncated {
        text.push_str("\n\n… (file truncated for preview)\n");
    }
    Some(text)
}
