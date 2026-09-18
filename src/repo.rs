//! Walk up to the git worktree root.

use std::path::{Path, PathBuf};

pub fn find_repo(start: &Path) -> Option<PathBuf> {
    let mut cur = std::fs::canonicalize(start).ok()?;
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_git_dir() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        fs::create_dir(tmp.path().join("src")).unwrap();
        let found = find_repo(&tmp.path().join("src")).unwrap();
        assert_eq!(found, std::fs::canonicalize(tmp.path()).unwrap());
    }

    #[test]
    fn none_without_git() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_repo(tmp.path()).is_none());
    }
}
