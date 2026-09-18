//! Ranked file-body packing. No reason, no include.

use std::collections::BTreeMap;
use std::path::Path;

use crate::paths::{canonicalize_in_repo, volume_is_case_insensitive};
use crate::session::{Omitted, SetPath, WorkingSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: String,
    pub reason: String,
    pub symbols: Vec<String>,
}

impl Candidate {
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
            symbols: Vec::new(),
        }
    }
}

pub fn pack(
    repo: &Path,
    task: &str,
    budget_tokens: u32,
    engine: &str,
    audit: Vec<String>,
    candidates: Vec<Candidate>,
) -> WorkingSet {
    pack_filtered(
        repo,
        task,
        budget_tokens,
        engine,
        audit,
        candidates,
        &[],
        &[],
    )
}

pub fn pack_filtered(
    repo: &Path,
    task: &str,
    budget_tokens: u32,
    engine: &str,
    mut audit: Vec<String>,
    candidates: Vec<Candidate>,
    include: &[String],
    exclude: &[String],
) -> WorkingSet {
    let mut set = WorkingSet::empty(task, budget_tokens);
    set.engine = engine.to_string();
    if !audit.iter().any(|a| a.starts_with("engine=")) {
        audit.insert(0, format!("engine={engine}"));
    }
    set.audit = audit;

    let casefold = volume_is_case_insensitive();
    let mut omitted: BTreeMap<String, u32> = BTreeMap::new();

    for cand in candidates {
        let posix = match canonicalize_in_repo(repo, &cand.path) {
            Ok(p) => p.as_posix().to_string(),
            Err(_) => {
                bump(&mut omitted, &cand.reason);
                continue;
            }
        };
        if !path_allowed(&posix, include, exclude) {
            bump(&mut omitted, &cand.reason);
            continue;
        }
        if set.contains_posix(&posix, casefold) {
            merge_symbols(&mut set, &posix, casefold, &cand.symbols);
            continue;
        }
        let abs = repo.join(posix.replace('/', std::path::MAIN_SEPARATOR_STR));
        let bytes = std::fs::read(&abs).ok().map(|b| b.len()).unwrap_or(0);
        let add = WorkingSet::approx_tokens(bytes);
        let over = set.used_tokens.saturating_add(add) > set.budget_tokens;
        if over && !set.paths.is_empty() {
            bump(&mut omitted, "budget");
            continue;
        }
        set.used_tokens = set.used_tokens.saturating_add(add);
        set.paths.push(SetPath {
            path: posix,
            reason: cand.reason,
            symbols: unique_symbols(&cand.symbols),
        });
    }

    set.omitted = omitted
        .into_iter()
        .map(|(reason, count)| Omitted { reason, count })
        .collect();
    set
}

pub fn path_allowed(posix: &str, include: &[String], exclude: &[String]) -> bool {
    if !include.is_empty() && !include.iter().any(|g| glob_match(g, posix)) {
        return false;
    }
    if exclude.iter().any(|g| glob_match(g, posix)) {
        return false;
    }
    true
}

/// POSIX glob: `*` is one segment, `**` is any path prefix/span.
pub fn glob_match(pat: &str, posix: &str) -> bool {
    let pat = pat.trim().trim_start_matches('/');
    let posix = posix.trim_start_matches('/');
    glob_rec(pat, posix)
}

fn glob_rec(pat: &str, path: &str) -> bool {
    if pat.is_empty() {
        return path.is_empty();
    }
    if let Some(rest) = pat.strip_prefix("**/") {
        if glob_rec(rest, path) {
            return true;
        }
        if let Some((_, tail)) = path.split_once('/') {
            return glob_rec(pat, tail);
        }
        return glob_rec(rest, "");
    }
    if pat == "**" {
        return true;
    }
    let (seg_pat, pat_rest) = match pat.split_once('/') {
        Some((a, b)) => (a, Some(b)),
        None => (pat, None),
    };
    let (seg_path, path_rest) = match path.split_once('/') {
        Some((a, b)) => (a, Some(b)),
        None => (path, None),
    };
    if !seg_match(seg_pat, seg_path) {
        return false;
    }
    match (pat_rest, path_rest) {
        (None, None) => true,
        (Some(p), Some(s)) => glob_rec(p, s),
        (None, Some(_)) => false,
        (Some(p), None) => p.is_empty() || p == "**" || p == "**/",
    }
}

fn seg_match(pat: &str, seg: &str) -> bool {
    if pat == "*" {
        return !seg.is_empty();
    }
    if !pat.contains('*') {
        return pat.eq_ignore_ascii_case(seg);
    }
    let parts: Vec<&str> = pat.split('*').collect();
    let mut cursor = seg;
    if !pat.starts_with('*') {
        let first = parts[0];
        if !cursor.to_ascii_lowercase().starts_with(&first.to_ascii_lowercase()) {
            return false;
        }
        cursor = &cursor[first.len()..];
    }
    if !pat.ends_with('*') {
        let last = *parts.last().unwrap_or(&"");
        if last.is_empty() {
            return true;
        }
        if !cursor.to_ascii_lowercase().ends_with(&last.to_ascii_lowercase()) {
            return false;
        }
    }
    true
}

fn bump(map: &mut BTreeMap<String, u32>, reason: &str) {
    *map.entry(reason.to_string()).or_insert(0) += 1;
}

fn unique_symbols(symbols: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for s in symbols {
        if !s.is_empty() && !out.iter().any(|e: &String| e.eq_ignore_ascii_case(s)) {
            out.push(s.clone());
        }
    }
    out
}

fn merge_symbols(set: &mut WorkingSet, posix: &str, casefold: bool, symbols: &[String]) {
    if let Some(p) = set.paths.iter_mut().find(|p| {
        if casefold {
            p.path.eq_ignore_ascii_case(posix)
        } else {
            p.path == posix
        }
    }) {
        for s in unique_symbols(symbols) {
            if !p.symbols.iter().any(|e| e.eq_ignore_ascii_case(&s)) {
                p.symbols.push(s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo_with(files: &[(&str, &str)]) -> tempfile::TempDir {
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

    #[test]
    fn packs_in_rank_order_and_omits_by_budget() {
        let tmp = repo_with(&[
            ("src/a.rs", "aaaa"),
            ("src/b.rs", "bbbbbbbb"),
            ("src/c.rs", "cccc"),
        ]);
        let set = pack(
            tmp.path(),
            "t",
            1,
            "diff",
            vec![],
            vec![
                Candidate::new("src/a.rs", "changed"),
                Candidate::new("src/b.rs", "callee"),
                Candidate::new("src/c.rs", "caller"),
            ],
        );
        assert_eq!(set.paths.len(), 1);
        assert_eq!(set.paths[0].path, "src/a.rs");
        assert_eq!(set.paths[0].reason, "changed");
        let budget = set.omitted.iter().find(|o| o.reason == "budget").unwrap();
        assert_eq!(budget.count, 2);
    }

    #[test]
    fn keeps_first_changed_path_even_if_over_budget() {
        let tmp = repo_with(&[("big.rs", &"x".repeat(100))]);
        let set = pack(
            tmp.path(),
            "t",
            1,
            "diff",
            vec![],
            vec![Candidate::new("big.rs", "changed")],
        );
        assert_eq!(set.paths.len(), 1);
        assert!(set.used_tokens > set.budget_tokens);
    }

    #[test]
    fn glob_starstar_and_exclude() {
        assert!(glob_match("src/**", "src/lib.rs"));
        assert!(glob_match("**/*.rs", "src/lib.rs"));
        assert!(!glob_match("tests/**", "src/lib.rs"));
        assert!(path_allowed("src/a.rs", &[], &["vendor/**".into()]));
        assert!(!path_allowed("vendor/x.rs", &[], &["vendor/**".into()]));
        assert!(!path_allowed("src/a.rs", &["tests/**".into()], &[]));
    }

    #[test]
    fn omitted_escape_counts_against_the_rank() {
        let tmp = repo_with(&[("keep.txt", "k")]);
        let set = pack(
            tmp.path(),
            "t",
            8000,
            "diff",
            vec![],
            vec![Candidate::new("../secret", "changed")],
        );
        assert!(set.paths.is_empty());
        assert_eq!(set.omitted[0].reason, "changed");
        assert_eq!(set.omitted[0].count, 1);
    }
}
