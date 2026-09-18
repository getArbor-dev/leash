//! Diff-only working set. v0. Arbor is a later engine swap.

use std::path::Path;
use std::process::Command;

use crate::paths::{canonicalize_in_repo, volume_is_case_insensitive};
use crate::session::{Omitted, SetPath, WorkingSet, DEFAULT_BUDGET_TOKENS};

#[derive(Debug)]
pub enum DiffError {
    GitNotFound,
    NotARepo,
    GitFailed(String),
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffError::GitNotFound => write!(f, "git not found on PATH"),
            DiffError::NotARepo => write!(f, "not a git repository"),
            DiffError::GitFailed(s) => write!(f, "git failed: {s}"),
        }
    }
}

impl std::error::Error for DiffError {}

pub fn changed_paths(repo: &Path) -> Result<Vec<String>, DiffError> {
    let output = Command::new("git")
        .args(["-C", repo.to_str().ok_or(DiffError::NotARepo)?, "diff", "--name-only", "-z", "HEAD"])
        .output()
        .map_err(|_| DiffError::GitNotFound)?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("not a git repository") {
            return Err(DiffError::NotARepo);
        }
        return Err(DiffError::GitFailed(err.trim().to_string()));
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    Ok(raw
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('\\', "/"))
        .collect())
}

pub fn build_from_diff(
    repo: &Path,
    task: &str,
    budget_tokens: u32,
    extra_paths: &[String],
) -> Result<WorkingSet, DiffError> {
    let mut set = WorkingSet::empty(task, budget_tokens);
    set.engine = "diff".to_string();
    set.audit.push("engine=diff".to_string());

    let mut ranked: Vec<(String, &'static str)> = Vec::new();
    for p in changed_paths(repo)? {
        ranked.push((p, "changed"));
    }
    for p in extra_paths {
        ranked.push((p.clone(), "expanded"));
    }

    let casefold = volume_is_case_insensitive();
    let mut omitted_budget = 0u32;

    for (raw, reason) in ranked {
        let posix = match canonicalize_in_repo(repo, &raw) {
            Ok(p) => p.as_posix().to_string(),
            Err(_) => {
                omitted_budget += 1;
                continue;
            }
        };
        if set.contains_posix(&posix, casefold) {
            continue;
        }
        let abs = repo.join(posix.replace('/', std::path::MAIN_SEPARATOR_STR));
        let bytes = std::fs::read(&abs).ok().map(|b| b.len()).unwrap_or(0);
        let add = WorkingSet::approx_tokens(bytes);
        if set.used_tokens.saturating_add(add) > set.budget_tokens && !set.paths.is_empty() {
            omitted_budget += 1;
            continue;
        }
        set.used_tokens = set.used_tokens.saturating_add(add);
        set.paths.push(SetPath {
            path: posix,
            reason: reason.to_string(),
            symbols: vec![],
        });
    }

    if omitted_budget > 0 {
        set.omitted.push(Omitted {
            reason: "budget".into(),
            count: omitted_budget,
        });
    }
    Ok(set)
}

pub fn default_budget() -> u32 {
    DEFAULT_BUDGET_TOKENS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn git_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        assert!(Command::new("git").args(["init"]).current_dir(p).status().unwrap().success());
        Command::new("git")
            .args(["config", "user.email", "leash@test"])
            .current_dir(p)
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "leash"])
            .current_dir(p)
            .status()
            .unwrap();
        fs::write(p.join("keep.txt"), "keep").unwrap();
        assert!(Command::new("git").args(["add", "keep.txt"]).current_dir(p).status().unwrap().success());
        assert!(Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "init"])
            .current_dir(p)
            .status()
            .unwrap()
            .success());
        tmp
    }

    #[test]
    fn empty_diff_yields_empty_set() {
        let tmp = git_repo();
        let set = build_from_diff(tmp.path(), "noop", 8000, &[]).unwrap();
        assert_eq!(set.engine, "diff");
        assert!(set.paths.is_empty());
    }

    #[test]
    fn modified_file_is_changed() {
        let tmp = git_repo();
        fs::write(tmp.path().join("keep.txt"), "changed").unwrap();
        let set = build_from_diff(tmp.path(), "edit keep", 8000, &[]).unwrap();
        assert_eq!(set.paths.len(), 1);
        assert_eq!(set.paths[0].path, "keep.txt");
        assert_eq!(set.paths[0].reason, "changed");
    }

    #[test]
    fn extra_path_is_expanded() {
        let tmp = git_repo();
        fs::write(tmp.path().join("other.txt"), "x").unwrap();
        let set = build_from_diff(
            tmp.path(),
            "seed",
            8000,
            &["other.txt".to_string()],
        )
        .unwrap();
        assert!(set.contains_posix("other.txt", cfg!(windows)));
        assert_eq!(
            set.paths.iter().find(|p| p.path == "other.txt").unwrap().reason,
            "expanded"
        );
    }
}
