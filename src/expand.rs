//! Explicit expand. Silent growth is a bug.

use std::path::Path;

use crate::arbor::{self, Probe, TIMEOUT};
use crate::pack::path_allowed;
use crate::paths::{canonicalize_in_repo, volume_is_case_insensitive};
use crate::session::{SetPath, WorkingSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandSpec {
    pub path: Option<String>,
    pub symbol: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandError {
    Uncanonical(String),
    NotCited,
    OverBudget { add: u32, used: u32, budget: u32 },
    NoTarget,
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::Uncanonical(e) => write!(f, "{e}"),
            ExpandError::NotCited => write!(
                f,
                "expand must cite a path or symbol already in the set, a user --path seed, or a graph neighbor"
            ),
            ExpandError::OverBudget { add, used, budget } => write!(
                f,
                "expand would use {add} more tokens ({used}/{budget}); raise budget_tokens in leash.yml"
            ),
            ExpandError::NoTarget => write!(f, "expand requires --path or --symbol"),
        }
    }
}

pub enum Neighbors {
    Arbor(Probe),
    Fixed(Vec<String>),
}

pub fn apply(
    repo: &Path,
    set: &mut WorkingSet,
    spec: &ExpandSpec,
    neighbors: &Neighbors,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<String>, ExpandError> {
    if spec.path.is_none() && spec.symbol.is_none() {
        return Err(ExpandError::NoTarget);
    }
    let casefold = volume_is_case_insensitive();
    let to_add = targets(repo, set, spec, neighbors, casefold)?;
    let mut new_files: Vec<(String, Vec<String>)> = Vec::new();
    for (posix, symbols) in to_add {
        if set.contains_posix(&posix, casefold) {
            continue;
        }
        if !path_allowed(&posix, include, exclude) {
            continue;
        }
        new_files.push((posix, symbols));
    }
    if new_files.is_empty() {
        let label = spec.path.as_deref().or(spec.symbol.as_deref()).unwrap_or("");
        set.audit.push(format!(
            "expand {label} tokens=+0 ({}) engine={}",
            spec.reason, set.engine
        ));
        return Ok(vec![]);
    }
    let mut add = 0u32;
    for (posix, _) in &new_files {
        let abs = repo.join(posix.replace('/', std::path::MAIN_SEPARATOR_STR));
        let bytes = std::fs::read(&abs).ok().map(|b| b.len()).unwrap_or(0);
        add = add.saturating_add(WorkingSet::approx_tokens(bytes));
    }
    if set.used_tokens.saturating_add(add) > set.budget_tokens {
        return Err(ExpandError::OverBudget {
            add,
            used: set.used_tokens,
            budget: set.budget_tokens,
        });
    }
    let mut added_paths = Vec::new();
    for (posix, symbols) in new_files {
        set.used_tokens = set.used_tokens.saturating_add({
            let abs = repo.join(posix.replace('/', std::path::MAIN_SEPARATOR_STR));
            WorkingSet::approx_tokens(std::fs::read(&abs).ok().map(|b| b.len()).unwrap_or(0))
        });
        added_paths.push(posix.clone());
        set.paths.push(SetPath {
            path: posix,
            reason: "expanded".into(),
            symbols,
        });
    }
    let label = spec
        .path
        .as_deref()
        .or(spec.symbol.as_deref())
        .unwrap_or("");
    set.audit.push(format!(
        "expand {label} tokens=+{add} ({}) engine={}",
        spec.reason, set.engine
    ));
    Ok(added_paths)
}

fn targets(
    repo: &Path,
    set: &WorkingSet,
    spec: &ExpandSpec,
    neighbors: &Neighbors,
    casefold: bool,
) -> Result<Vec<(String, Vec<String>)>, ExpandError> {
    if let Some(raw) = &spec.path {
        let posix = canonicalize_in_repo(repo, raw)
            .map_err(|e| ExpandError::Uncanonical(e.to_string()))?
            .as_posix()
            .to_string();
        if set.contains_posix(&posix, casefold) || is_seed(repo, set, &posix, casefold) {
            return Ok(vec![(posix, vec![])]);
        }
        let neigh = neighbor_set(repo, set, neighbors);
        if neigh.iter().any(|p| {
            if casefold {
                p.eq_ignore_ascii_case(&posix)
            } else {
                p == &posix
            }
        }) {
            return Ok(vec![(posix, vec![])]);
        }
        return Err(ExpandError::NotCited);
    }
    let symbol = spec.symbol.as_deref().unwrap();
    if !set.contains_symbol(symbol) {
        return Err(ExpandError::NotCited);
    }
    let files = neighbor_files_for_symbol(repo, symbol, neighbors);
    if files.is_empty() {
        return Err(ExpandError::NotCited);
    }
    Ok(files.into_iter().map(|p| (p, vec![symbol.to_string()])).collect())
}

fn is_seed(repo: &Path, set: &WorkingSet, posix: &str, casefold: bool) -> bool {
    set.audit.iter().filter_map(|a| a.strip_prefix("seed ")).any(|raw| {
        if let Ok(p) = canonicalize_in_repo(repo, raw) {
            if casefold {
                p.as_posix().eq_ignore_ascii_case(posix)
            } else {
                p.as_posix() == posix
            }
        } else if casefold {
            raw.eq_ignore_ascii_case(posix)
        } else {
            raw == posix
        }
    })
}

fn neighbor_set(repo: &Path, set: &WorkingSet, neighbors: &Neighbors) -> Vec<String> {
    match neighbors {
        Neighbors::Fixed(v) => v.clone(),
        Neighbors::Arbor(probe) => {
            let mut out = Vec::new();
            for p in &set.paths {
                for sym in &p.symbols {
                    out.extend(neighbor_files_for_symbol(repo, sym, neighbors));
                }
            }
            let _ = probe;
            out
        }
    }
}

fn neighbor_files_for_symbol(repo: &Path, symbol: &str, neighbors: &Neighbors) -> Vec<String> {
    match neighbors {
        Neighbors::Fixed(v) => v.clone(),
        Neighbors::Arbor(probe) => {
            let mut files = Vec::new();
            for kind in ["callees", "callers"] {
                if let Ok(raw) = arbor::run_json(probe, repo, &[kind, symbol, ".", "--json"], TIMEOUT) {
                    if let Ok(v) = serde_json::from_str(&raw) {
                        for (_, path) in arbor::parse_named_files(&v) {
                            if !path.is_empty() {
                                files.push(path);
                            }
                        }
                    }
                }
            }
            files
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::WorkingSet;
    use std::fs;

    fn repo_files(files: &[(&str, &str)]) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        for (rel, body) in files {
            let path = tmp.path().join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, body).unwrap();
        }
        tmp
    }

    fn set_with(paths: &[(&str, &str)], budget: u32) -> WorkingSet {
        let mut set = WorkingSet::empty("t", budget);
        for (p, reason) in paths {
            set.paths.push(SetPath {
                path: (*p).into(),
                reason: (*reason).into(),
                symbols: vec!["login".into()],
            });
        }
        set
    }

    #[test]
    fn stranger_is_not_cited() {
        let tmp = repo_files(&[("src/a.rs", "a"), ("src/b.rs", "b")]);
        let mut set = set_with(&[("src/a.rs", "changed")], 8000);
        let err = apply(
            tmp.path(),
            &mut set,
            &ExpandSpec {
                path: Some("src/b.rs".into()),
                symbol: None,
                reason: "nope".into(),
            },
            &Neighbors::Fixed(vec![]),
            &[],
            &[],
        )
        .unwrap_err();
        assert_eq!(err, ExpandError::NotCited);
    }

    #[test]
    fn neighbor_path_is_expanded() {
        let tmp = repo_files(&[("src/a.rs", "a"), ("src/b.rs", "bb")]);
        let mut set = set_with(&[("src/a.rs", "changed")], 8000);
        let added = apply(
            tmp.path(),
            &mut set,
            &ExpandSpec {
                path: Some("src/b.rs".into()),
                symbol: None,
                reason: "callee of login".into(),
            },
            &Neighbors::Fixed(vec!["src/b.rs".into()]),
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(added, vec!["src/b.rs"]);
        assert!(set.contains_posix("src/b.rs", cfg!(windows)));
        assert!(set.audit.iter().any(|a| a.contains("tokens=+")));
    }

    #[test]
    fn user_seed_is_cited() {
        let tmp = repo_files(&[("src/a.rs", "a"), ("seeded.rs", "s")]);
        let mut set = set_with(&[("src/a.rs", "changed")], 8000);
        set.audit.push("seed seeded.rs".into());
        apply(
            tmp.path(),
            &mut set,
            &ExpandSpec {
                path: Some("seeded.rs".into()),
                symbol: None,
                reason: "user asked".into(),
            },
            &Neighbors::Fixed(vec![]),
            &[],
            &[],
        )
        .unwrap();
        assert!(set.contains_posix("seeded.rs", cfg!(windows)));
    }

    #[test]
    fn over_budget_fails_closed() {
        let tmp = repo_files(&[("src/a.rs", "a"), ("src/b.rs", &"x".repeat(100))]);
        let mut set = set_with(&[("src/a.rs", "changed")], 2);
        set.used_tokens = 2;
        let err = apply(
            tmp.path(),
            &mut set,
            &ExpandSpec {
                path: Some("src/b.rs".into()),
                symbol: None,
                reason: "need it".into(),
            },
            &Neighbors::Fixed(vec!["src/b.rs".into()]),
            &[],
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, ExpandError::OverBudget { .. }));
        assert!(!set.contains_posix("src/b.rs", cfg!(windows)));
    }

    #[test]
    fn symbol_requires_in_set_symbol() {
        let tmp = repo_files(&[("src/a.rs", "a")]);
        let mut set = WorkingSet::empty("t", 8000);
        set.paths.push(SetPath {
            path: "src/a.rs".into(),
            reason: "changed".into(),
            symbols: vec![],
        });
        let err = apply(
            tmp.path(),
            &mut set,
            &ExpandSpec {
                path: None,
                symbol: Some("login".into()),
                reason: "trace".into(),
            },
            &Neighbors::Fixed(vec!["src/b.rs".into()]),
            &[],
            &[],
        )
        .unwrap_err();
        assert_eq!(err, ExpandError::NotCited);
    }
}
