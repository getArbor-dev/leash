//! Command implementations used by the `leash` binary.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::arbor::Probe;
use crate::cli::{help_text, Cmd};
use crate::config;
use crate::engine;
use crate::hook::{decide, to_claude_json, HookInput};
use crate::repo::find_repo;
use crate::session::WorkingSet;
use crate::store;

pub fn run(args: &[String], stdin: &mut dyn Read, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let cmd = match crate::cli::parse(args) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            let _ = write!(stderr, "{}", help_text());
            return 2;
        }
    };
    match cmd {
        Cmd::Help => {
            let _ = write!(stdout, "{}", help_text());
            0
        }
        Cmd::Session { task, paths } => cmd_session(&task, &paths, stdin, stdout, stderr),
        Cmd::SessionFromHook => cmd_session_from_hook(stdin, stdout, stderr),
        Cmd::Hook => cmd_hook(stdin, stdout, stderr),
        Cmd::Status => cmd_status(stdout, stderr),
        Cmd::Expand { path, symbol, reason } => cmd_expand(path, symbol, &reason, stdout, stderr),
        Cmd::Install => cmd_install(stdout, stderr),
        Cmd::Ci { base, task } => cmd_ci(&base, &task, stdout, stderr),
    }
}

fn cwd_repo() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|_| "no cwd".to_string())?;
    find_repo(&cwd).ok_or_else(|| "not a git repository".to_string())
}

fn cmd_session(
    task: &str,
    paths: &[String],
    _stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let repo = match cwd_repo() {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            return 2;
        }
    };
    match start_session(&repo, task, paths, &Probe::from_env()) {
        Ok(set) => {
            let _ = writeln!(stdout, "{}", format_enforcing(&set));
            0
        }
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            2
        }
    }
}

fn start_session(repo: &Path, task: &str, extra: &[String], probe: &Probe) -> Result<WorkingSet, String> {
    let cfg = config::load(repo);
    let set = engine::build(repo, task, extra, &cfg, probe).map_err(|e| e.to_string())?;
    store::save(repo, &set).map_err(|e| e.to_string())?;
    Ok(set)
}

fn cmd_session_from_hook(stdin: &mut dyn Read, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let mut raw = String::new();
    let _ = stdin.read_to_string(&mut raw);
    let input: Value = serde_json::from_str(&raw).unwrap_or(json!({}));
    let start = input
        .get("cwd")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let Some(repo) = find_repo(&start) else {
        let _ = writeln!(
            stdout,
            "{}",
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": "leash: NOT ENFORCING (not a git repository)"
                }
            })
        );
        return 0;
    };
    match start_session(&repo, "session", &[], &Probe::from_env()) {
        Ok(set) => {
            let ctx = format_enforcing(&set);
            let _ = writeln!(
                stdout,
                "{}",
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": ctx
                    }
                })
            );
            0
        }
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            let _ = writeln!(
                stdout,
                "{}",
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": format!("leash: NOT ENFORCING ({e})")
                    }
                })
            );
            0
        }
    }
}

fn cmd_hook(stdin: &mut dyn Read, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let mut raw = String::new();
    if stdin.read_to_string(&mut raw).is_err() {
        return emit_deny(stdout, stderr, "could not read hook stdin");
    }
    let input: HookInput = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(_) => return emit_deny(stdout, stderr, "hook stdin is not JSON"),
    };
    let start = input
        .cwd
        .as_deref()
        .map(Path::new)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let Some(repo) = find_repo(&start) else {
        return emit_deny(stdout, stderr, "not a git repository");
    };
    let set = store::load(&repo);
    let decision = decide(&repo, set.as_ref(), &input);
    let json = to_claude_json(&decision);
    let _ = writeln!(stdout, "{json}");
    if !decision.allow {
        let _ = writeln!(stderr, "{}", decision.one_line());
    }
    0
}

fn emit_deny(stdout: &mut dyn Write, stderr: &mut dyn Write, msg: &str) -> i32 {
    let d = crate::decision::Decision::deny(
        crate::decision::RuleClass::PathUncanonical,
        None,
        None,
        msg,
    );
    let _ = writeln!(stdout, "{}", to_claude_json(&d));
    let _ = writeln!(stderr, "{}", d.one_line());
    2
}

fn cmd_status(stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let repo = match cwd_repo() {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(stdout, "leash: NOT ENFORCING ({e})");
            return 0;
        }
    };
    match store::load(&repo) {
        Some(set) => {
            let _ = writeln!(stdout, "{}", format_enforcing(&set));
            0
        }
        None => {
            let _ = writeln!(stdout, "leash: NOT ENFORCING (no session)");
            let _ = writeln!(stderr, "run: leash session --task \"...\"");
            0
        }
    }
}

fn cmd_expand(
    path: Option<String>,
    symbol: Option<String>,
    reason: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let repo = match cwd_repo() {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            return 2;
        }
    };
    let Some(mut set) = store::load(&repo) else {
        let _ = writeln!(stderr, "leash: no session; run leash session --task \"...\"");
        return 2;
    };
    let cfg = config::load(&repo);
    let spec = crate::expand::ExpandSpec {
        path,
        symbol,
        reason: reason.to_string(),
    };
    match crate::expand::apply(
        &repo,
        &mut set,
        &spec,
        &crate::expand::Neighbors::Arbor(Probe::from_env()),
        &cfg.include,
        &cfg.exclude,
    ) {
        Ok(added) => {
            if store::save(&repo, &set).is_err() {
                let _ = writeln!(stderr, "leash: could not write session");
                return 2;
            }
            if added.is_empty() {
                let _ = writeln!(stdout, "expand: already in set");
            } else {
                for p in added {
                    let _ = writeln!(stdout, "expanded {p}");
                }
            }
            0
        }
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            2
        }
    }
}

fn cmd_ci(base: &str, task: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let repo = match cwd_repo() {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            return 2;
        }
    };
    match crate::ci::run(&repo, base, task) {
        Ok(report) => {
            let _ = write!(stdout, "{}", crate::ci::markdown(&report));
            crate::ci::exit_code(&report)
        }
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            2
        }
    }
}

const CLAUDE_SETTINGS: &str = include_str!("../contrib/claude.settings.json");

fn cmd_install(stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let repo = match cwd_repo() {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(stderr, "leash: {e}");
            return 2;
        }
    };
    let dir = repo.join(".claude");
    let path = dir.join("settings.json");
    if path.exists() {
        let _ = writeln!(
            stderr,
            "leash: {} already exists; merge hooks by hand from `leash install` output",
            path.display()
        );
        let _ = write!(stdout, "{CLAUDE_SETTINGS}");
        return 2;
    }
    if std::fs::create_dir_all(&dir).is_err() || std::fs::write(&path, CLAUDE_SETTINGS).is_err() {
        let _ = writeln!(stderr, "leash: could not write {}", path.display());
        return 2;
    }
    let _ = writeln!(stdout, "wrote {}", path.display());
    0
}

fn format_enforcing(set: &WorkingSet) -> String {
    let mut out = format!(
        "leash: enforcing\nbudget {}/{}\nengine {}\ntask {}\n",
        set.used_tokens, set.budget_tokens, set.engine, set.task
    );
    for p in &set.paths {
        out.push_str(&format!("  {} ({})\n", p.path, p.reason));
    }
    if set.paths.is_empty() {
        out.push_str("  (empty set — writes deny until expand or a git diff)\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn git_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        Command::new("git").args(["init"]).current_dir(p).status().unwrap();
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
        Command::new("git").args(["add", "keep.txt"]).current_dir(p).status().unwrap();
        Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "init"])
            .current_dir(p)
            .status()
            .unwrap();
        tmp
    }

    #[test]
    fn start_session_writes_json() {
        let tmp = git_repo();
        fs::write(tmp.path().join("keep.txt"), "changed").unwrap();
        let set = start_session(tmp.path(), "edit", &[], &Probe::Off).unwrap();
        assert_eq!(set.engine, "diff");
        assert!(store::load(tmp.path()).is_some());
        assert_eq!(set.paths[0].path, "keep.txt");
    }
}
