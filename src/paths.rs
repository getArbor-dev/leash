//! Repo-relative path canonicalization.
//!
//! Output is POSIX, no leading slash. Membership against the working set
//! uses this string, case-folded on case-insensitive volumes.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPath {
    posix: String,
}

impl RepoPath {
    pub fn as_posix(&self) -> &str {
        &self.posix
    }

    pub fn matches(&self, other: &RepoPath, case_insensitive: bool) -> bool {
        if case_insensitive {
            self.posix.eq_ignore_ascii_case(&other.posix)
        } else {
            self.posix == other.posix
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    Empty,
    NulOrControl,
    EscapesRepo,
    Io,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::Empty => write!(f, "empty path"),
            PathError::NulOrControl => write!(f, "NUL or control byte in path"),
            PathError::EscapesRepo => write!(f, "path escapes the repository"),
            PathError::Io => write!(f, "path could not be resolved"),
        }
    }
}

impl std::error::Error for PathError {}

pub fn volume_is_case_insensitive() -> bool {
    cfg!(windows)
}

/// Resolve `raw` to a repo-relative POSIX path under `repo_root`.
///
/// Follows existing symlinks via `fs::canonicalize`. A symlink that
/// resolves outside the repo is `EscapesRepo` (hook maps this to
/// `PATH_UNCANONICAL`).
pub fn canonicalize_in_repo(repo_root: &Path, raw: &str) -> Result<RepoPath, PathError> {
    if raw.is_empty() {
        return Err(PathError::Empty);
    }
    if raw.bytes().any(|b| b == 0 || b < 32) {
        return Err(PathError::NulOrControl);
    }

    let repo_root = std::fs::canonicalize(repo_root).map_err(|_| PathError::Io)?;
    let candidate = candidate_path(&repo_root, raw)?;
    let resolved = resolve_existing_prefix(&candidate)?;

    let rel = strip_repo_prefix(&repo_root, &resolved)?;
    let mut posix = rel
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    if posix.is_empty() || posix.split('/').any(|p| p == "..") {
        return Err(PathError::EscapesRepo);
    }
    if volume_is_case_insensitive() {
        posix = posix.to_ascii_lowercase();
    }
    Ok(RepoPath { posix })
}

fn candidate_path(repo_root: &Path, raw: &str) -> Result<PathBuf, PathError> {
    let normalized = raw.replace('\\', "/");
    let as_path = Path::new(&normalized);
    if as_path.is_absolute() {
        return Ok(as_path.to_path_buf());
    }

    let mut out = PathBuf::from(repo_root);
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if !out.pop() || !out.starts_with(repo_root) {
                return Err(PathError::EscapesRepo);
            }
            continue;
        }
        out.push(part);
    }
    Ok(out)
}

/// Canonicalize the longest existing prefix, then append the rest.
/// This lets Write to a new file still get symlink-resolved parents.
fn resolve_existing_prefix(path: &Path) -> Result<PathBuf, PathError> {
    if path.exists() {
        return std::fs::canonicalize(path).map_err(|_| PathError::Io);
    }

    let mut suffix: Vec<std::ffi::OsString> = Vec::new();
    let mut cursor = path.to_path_buf();
    loop {
        if cursor.exists() {
            let mut resolved = std::fs::canonicalize(&cursor).map_err(|_| PathError::Io)?;
            for part in suffix.iter().rev() {
                resolved.push(part);
            }
            return Ok(resolved);
        }
        match cursor.file_name() {
            Some(name) => {
                suffix.push(name.to_os_string());
                if !cursor.pop() {
                    return Err(PathError::Io);
                }
            }
            None => return Err(PathError::Io),
        }
    }
}

fn strip_repo_prefix(repo_root: &Path, resolved: &Path) -> Result<PathBuf, PathError> {
    if let Ok(rel) = resolved.strip_prefix(repo_root) {
        if rel.as_os_str().is_empty() {
            return Err(PathError::Empty);
        }
        return Ok(rel.to_path_buf());
    }
    if volume_is_case_insensitive() {
        let repo_s = repo_root
            .to_string_lossy()
            .to_ascii_lowercase()
            .replace('\\', "/");
        let res_s = resolved
            .to_string_lossy()
            .to_ascii_lowercase()
            .replace('\\', "/");
        let repo_s = repo_s.trim_end_matches('/');
        if let Some(rest) = res_s.strip_prefix(repo_s) {
            let rest = rest.trim_start_matches('/');
            if rest.is_empty() {
                return Err(PathError::Empty);
            }
            return Ok(PathBuf::from(rest));
        }
    }
    Err(PathError::EscapesRepo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn rejects_empty() {
        let tmp = repo();
        assert_eq!(
            canonicalize_in_repo(tmp.path(), "").unwrap_err(),
            PathError::Empty
        );
    }

    #[test]
    fn rejects_nul() {
        let tmp = repo();
        assert_eq!(
            canonicalize_in_repo(tmp.path(), "foo\0bar").unwrap_err(),
            PathError::NulOrControl
        );
    }

    #[test]
    fn rejects_newline() {
        let tmp = repo();
        assert_eq!(
            canonicalize_in_repo(tmp.path(), "foo\nbar").unwrap_err(),
            PathError::NulOrControl
        );
    }

    #[test]
    fn relative_file_inside_repo() {
        let tmp = repo();
        fs::write(tmp.path().join("a.txt"), "x").unwrap();
        let p = canonicalize_in_repo(tmp.path(), "a.txt").unwrap();
        assert_eq!(p.as_posix(), "a.txt");
    }

    #[test]
    fn nested_and_dot_segments() {
        let tmp = repo();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src").join("lib.rs"), "").unwrap();
        let p = canonicalize_in_repo(tmp.path(), "./src/./lib.rs").unwrap();
        assert_eq!(p.as_posix(), "src/lib.rs");
    }

    #[test]
    fn parent_segment_cannot_leave_repo() {
        let tmp = repo();
        assert_eq!(
            canonicalize_in_repo(tmp.path(), "../secret").unwrap_err(),
            PathError::EscapesRepo
        );
    }

    #[test]
    fn new_file_under_existing_parent() {
        let tmp = repo();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let p = canonicalize_in_repo(tmp.path(), "src/new.rs").unwrap();
        assert_eq!(p.as_posix(), "src/new.rs");
    }

    #[test]
    fn absolute_path_inside_repo() {
        let tmp = repo();
        fs::write(tmp.path().join("a.txt"), "x").unwrap();
        let abs = tmp.path().join("a.txt");
        let p = canonicalize_in_repo(tmp.path(), abs.to_str().unwrap()).unwrap();
        assert_eq!(p.as_posix(), "a.txt");
    }

    #[test]
    fn absolute_path_outside_repo() {
        let tmp = repo();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("x.txt"), "x").unwrap();
        let abs = outside.path().join("x.txt");
        assert_eq!(
            canonicalize_in_repo(tmp.path(), abs.to_str().unwrap()).unwrap_err(),
            PathError::EscapesRepo
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escaping_repo_is_denied() {
        let tmp = repo();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "s").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            tmp.path().join("link.txt"),
        )
        .unwrap();
        assert_eq!(
            canonicalize_in_repo(tmp.path(), "link.txt").unwrap_err(),
            PathError::EscapesRepo
        );
    }
}
