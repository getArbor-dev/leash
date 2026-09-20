//! Diff-only working set. Ranked packing lives in `pack`.

use std::path::Path;
use std::process::Command;

use crate::pack::{pack, Candidate};
use crate::session::WorkingSet;

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
    changed_against(repo, "HEAD")
}

/// `rev` is a git revision or range (`HEAD`, `main...HEAD`). Rejects
/// flag-shaped and whitespace values so they cannot become extra argv.
pub fn validate_rev(rev: &str) -> Result<(), DiffError> {
    if rev.is_empty()
        || rev.starts_with('-')
        || rev.chars().any(|c| c.is_whitespace() || c == '\0')
    {
        return Err(DiffError::GitFailed("invalid revision".into()));
    }
    Ok(())
}

pub fn changed_against(repo: &Path, rev: &str) -> Result<Vec<String>, DiffError> {
    validate_rev(rev)?;
    let output = git(repo, &["diff", "--name-only", "-z", rev])?;
    let raw = String::from_utf8_lossy(&output);
    Ok(raw
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('\\', "/"))
        .collect())
}

/// Added lines per path from `git diff -U0 <rev>`. Order is first-seen path.
pub fn added_by_path(repo: &Path, rev: &str) -> Result<Vec<(String, String)>, DiffError> {
    validate_rev(rev)?;
    let output = git(repo, &["diff", "-U0", "--no-color", "--find-renames", rev])?;
    let raw = String::from_utf8_lossy(&output);
    Ok(parse_unified_added(&raw))
}

fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, DiffError> {
    let repo_s = repo.to_str().ok_or(DiffError::NotARepo)?;
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_s);
    cmd.args(args);
    let output = cmd.output().map_err(|_| DiffError::GitNotFound)?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("not a git repository") {
            return Err(DiffError::NotARepo);
        }
        return Err(DiffError::GitFailed(err.trim().to_string()));
    }
    Ok(output.stdout)
}

pub(crate) fn parse_unified_added(diff: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<usize> = None;
    for line in diff.lines() {
        if let Some(path) = plus_plus_plus_path(line) {
            if path == "/dev/null" {
                current = None;
                continue;
            }
            if let Some(i) = out.iter().position(|(p, _)| p == &path) {
                current = Some(i);
            } else {
                out.push((path, String::new()));
                current = Some(out.len() - 1);
            }
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            if let Some(i) = current {
                let body = &mut out[i].1;
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(&line[1..]);
            }
        }
    }
    out.retain(|(_, added)| !added.is_empty());
    out
}

fn plus_plus_plus_path(line: &str) -> Option<String> {
    let rest = line.strip_prefix("+++ ")?;
    let rest = rest.strip_prefix("b/").unwrap_or(rest);
    let rest = rest.split('\t').next().unwrap_or(rest).trim();
    if rest.is_empty() {
        return None;
    }
    Some(rest.trim_matches('"').replace('\\', "/"))
}

pub fn build_from_diff(
    repo: &Path,
    task: &str,
    budget_tokens: u32,
    extra_paths: &[String],
) -> Result<WorkingSet, DiffError> {
    let mut candidates: Vec<Candidate> = changed_paths(repo)?
        .into_iter()
        .map(|p| Candidate::new(p, "changed"))
        .collect();
    for p in extra_paths {
        candidates.push(Candidate::new(p.clone(), "expanded"));
    }
    Ok(pack(
        repo,
        task,
        budget_tokens,
        "diff",
        vec!["engine=diff".into()],
        candidates,
    ))
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
    fn range_lists_committed_change() {
        let tmp = git_repo();
        let p = tmp.path();
        fs::write(p.join("keep.txt"), "changed").unwrap();
        assert!(Command::new("git").args(["add", "keep.txt"]).current_dir(p).status().unwrap().success());
        assert!(Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "edit"])
            .current_dir(p)
            .status()
            .unwrap()
            .success());
        let paths = changed_against(p, "HEAD~1...HEAD").unwrap();
        assert_eq!(paths, vec!["keep.txt".to_string()]);
        assert!(validate_rev("--all").is_err());
    }

    #[test]
    fn extra_path_is_expanded() {
        let tmp = git_repo();
        fs::write(tmp.path().join("other.txt"), "x").unwrap();
        let set = build_from_diff(tmp.path(), "seed", 8000, &["other.txt".to_string()]).unwrap();
        assert!(set.contains_posix("other.txt", cfg!(windows)));
        assert_eq!(
            set.paths.iter().find(|p| p.path == "other.txt").unwrap().reason,
            "expanded"
        );
    }
}
