//! Choose Arbor radius or the git-diff fallback. Label the one that ran.

use std::collections::HashSet;
use std::path::Path;

use crate::arbor::{self, ArborRadius, Probe, TIMEOUT};
use crate::config::Config;
use crate::diff::{self, DiffError};
use crate::pack::{pack_filtered, Candidate};
use crate::session::WorkingSet;

pub fn build(
    repo: &Path,
    task: &str,
    extra_paths: &[String],
    cfg: &Config,
    probe: &Probe,
) -> Result<WorkingSet, DiffError> {
    let git_changed = diff::changed_paths(repo)?;
    match probe {
        Probe::Off => Ok(assemble(
            repo,
            task,
            extra_paths,
            cfg,
            git_changed,
            None,
            vec!["engine=diff".into()],
        )),
        _ => match arbor::collect(repo, probe, TIMEOUT) {
            Ok(radius) => {
                Ok(assemble(
                    repo,
                    task,
                    extra_paths,
                    cfg,
                    git_changed,
                    Some(radius),
                    vec!["engine=arbor".into()],
                ))
            }
            Err(e) => {
                let audit = vec![
                    "engine=diff".into(),
                    format!("{} ({e})", e.audit_tag()),
                ];
                Ok(assemble(repo, task, extra_paths, cfg, git_changed, None, audit))
            }
        },
    }
}

pub fn assemble(
    repo: &Path,
    task: &str,
    extra_paths: &[String],
    cfg: &Config,
    git_changed: Vec<String>,
    radius: Option<ArborRadius>,
    mut audit: Vec<String>,
) -> WorkingSet {
    let engine = if radius.is_some() { "arbor" } else { "diff" };
    let mut seen: HashSet<String> = HashSet::new();
    let mut candidates: Vec<Candidate> = Vec::new();

    let mut changed = git_changed;
    if let Some(r) = radius.as_ref() {
        for f in &r.changed_files {
            if !changed.iter().any(|c| c.eq_ignore_ascii_case(f)) {
                changed.push(f.clone());
            }
        }
    }

    for path in &changed {
        let mut cand = Candidate::new(path, "changed");
        if let Some(r) = radius.as_ref() {
            cand.symbols = r
                .changed_symbols
                .iter()
                .filter(|(_, p)| p.eq_ignore_ascii_case(path))
                .map(|(n, _)| n.clone())
                .filter(|n| !n.is_empty())
                .collect();
        }
        remember(&mut seen, path);
        candidates.push(cand);
    }

    for p in extra_paths {
        audit.push(format!("seed {p}"));
        if remember(&mut seen, p) {
            continue;
        }
        candidates.push(Candidate::new(p, "expanded"));
    }

    if let Some(r) = radius.as_ref() {
        let mut tests: Vec<Candidate> = Vec::new();
        push_neighbors(&mut candidates, &mut tests, &mut seen, &r.callees, "callee");
        push_neighbors(&mut candidates, &mut tests, &mut seen, &r.callers, "caller");
        candidates.extend(tests);
    }

    if changed.iter().any(|p| is_manifest(p)) {
        for p in &changed {
            if let Some(lock) = lockfile_for(p) {
                let dir = p.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
                let lock_path = if dir.is_empty() {
                    lock.to_string()
                } else {
                    format!("{dir}/{lock}")
                };
                if !remember(&mut seen, &lock_path) {
                    candidates.push(Candidate::new(lock_path, "config"));
                }
            }
        }
    }

    pack_filtered(
        repo,
        task,
        cfg.budget_tokens,
        engine,
        audit,
        candidates,
        &cfg.include,
        &cfg.exclude,
    )
}

fn push_neighbors(
    dest: &mut Vec<Candidate>,
    tests: &mut Vec<Candidate>,
    seen: &mut HashSet<String>,
    neighbors: &[arbor::Neighbor],
    reason: &str,
) {
    for n in neighbors {
        if remember(seen, &n.path) {
            continue;
        }
        let mut cand = Candidate::new(&n.path, reason);
        cand.symbols = n.symbols.clone();
        if is_test_path(&n.path) {
            cand.reason = "test".into();
            tests.push(cand);
        } else {
            dest.push(cand);
        }
    }
}

fn remember(seen: &mut HashSet<String>, path: &str) -> bool {
    let key = path.replace('\\', "/").to_ascii_lowercase();
    !seen.insert(key)
}

pub fn is_test_path(posix: &str) -> bool {
    let p = posix.replace('\\', "/").to_ascii_lowercase();
    p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("/spec/")
        || p.contains("_test.")
        || p.contains(".test.")
        || p.contains(".spec.")
        || p.contains("_spec.")
        || p.ends_with("_test.rs")
}

pub fn is_manifest(posix: &str) -> bool {
    let name = posix
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or(posix)
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "package.json" | "cargo.toml" | "pyproject.toml" | "requirements.txt" | "go.mod" | "gemfile" | "composer.json"
    )
}

fn lockfile_for(posix: &str) -> Option<&'static str> {
    let name = posix
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or(posix)
        .to_ascii_lowercase();
    match name.as_str() {
        "package.json" => Some("package-lock.json"),
        "cargo.toml" => Some("Cargo.lock"),
        "pyproject.toml" | "requirements.txt" => Some("uv.lock"),
        "go.mod" => Some("go.sum"),
        "gemfile" => Some("Gemfile.lock"),
        "composer.json" => Some("composer.lock"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arbor::{parse_named_files, radius_from_diff, Neighbor};
    use std::fs;

    fn load_json(name: &str) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(format!("fixtures/arbor/{name}")).unwrap()).unwrap()
    }

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

    #[test]
    fn arbor_fixture_ranks_callee_caller_and_test() {
        let tmp = repo_files(&[
            ("src/auth.rs", "login"),
            ("src/crypto.rs", "hash"),
            ("src/http.rs", "handle"),
            ("tests/auth_test.rs", "test"),
        ]);
        let mut radius = radius_from_diff(&load_json("diff.json"));
        for (name, path) in parse_named_files(&load_json("file-graph.json")) {
            if !name.is_empty() {
                radius.changed_symbols.push((name, path));
            }
        }
        for (name, path) in parse_named_files(&load_json("callees.json")) {
            radius.callees.push(Neighbor {
                path,
                symbols: if name.is_empty() { vec![] } else { vec![name] },
            });
        }
        for (name, path) in parse_named_files(&load_json("callers.json")) {
            radius.callers.push(Neighbor {
                path,
                symbols: if name.is_empty() { vec![] } else { vec![name] },
            });
        }
        let set = assemble(
            tmp.path(),
            "login flow",
            &[],
            &Config::default(),
            vec!["src/auth.rs".into()],
            Some(radius),
            vec!["engine=arbor".into()],
        );
        assert_eq!(set.engine, "arbor");
        let reasons: Vec<_> = set.paths.iter().map(|p| (p.path.as_str(), p.reason.as_str())).collect();
        assert!(reasons.contains(&("src/auth.rs", "changed")));
        assert!(reasons.contains(&("src/crypto.rs", "callee")));
        assert!(reasons.contains(&("src/http.rs", "caller")));
        assert!(reasons.contains(&("tests/auth_test.rs", "test")));
        assert!(set.paths.iter().any(|p| p.path == "src/auth.rs" && p.symbols.iter().any(|s| s == "login")));
    }

    #[test]
    fn missing_arbor_stays_diff() {
        let tmp = repo_files(&[("keep.txt", "k")]);
        // not a git repo: changed_paths would fail. assemble with empty git list.
        let set = assemble(
            tmp.path(),
            "t",
            &[],
            &Config::default(),
            vec!["keep.txt".into()],
            None,
            vec!["engine=diff".into(), "arbor_missing (arbor not found on PATH)".into()],
        );
        assert_eq!(set.engine, "diff");
        assert_eq!(set.paths[0].path, "keep.txt");
        assert!(set.audit.iter().any(|a| a.contains("arbor_missing") || a == "engine=diff"));
    }
}
