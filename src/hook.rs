//! PreToolUse: deny Edit/Write/mutating Bash.

use serde::Deserialize;
use serde_json::{json, Value};

use crate::decision::{Decision, RuleClass};
use crate::paths::{canonicalize_in_repo, volume_is_case_insensitive, PathError};
use crate::rules::scan_hunk;
use crate::session::WorkingSet;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct HookInput {
    #[serde(default)]
    pub hook_event_name: String,
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Value,
    #[serde(default)]
    pub cwd: Option<String>,
}

pub fn decide(repo: &Path, set: Option<&WorkingSet>, input: &HookInput) -> Decision {
    if input.hook_event_name != "PreToolUse" && !input.hook_event_name.is_empty() {
        return Decision::allow();
    }
    match input.tool_name.as_str() {
        "Write" | "Edit" => decide_file(repo, set, input),
        "Bash" | "PowerShell" => decide_shell(repo, set, input),
        _ => Decision::allow(),
    }
}

fn decide_file(repo: &Path, set: Option<&WorkingSet>, input: &HookInput) -> Decision {
    let Some(raw) = input.tool_input.get("file_path").and_then(Value::as_str) else {
        return Decision::deny(
            RuleClass::PathUncanonical,
            None,
            None,
            "missing file_path",
        );
    };
    let posix = match canonicalize_in_repo(repo, raw) {
        Ok(p) => p,
        Err(PathError::EscapesRepo) | Err(PathError::NulOrControl) | Err(PathError::Empty) => {
            return Decision::deny(
                RuleClass::PathUncanonical,
                None,
                None,
                "path is not a canonical repo path",
            );
        }
        Err(PathError::Io) => {
            return Decision::deny(
                RuleClass::PathUncanonical,
                None,
                None,
                "path could not be resolved",
            );
        }
    };
    let Some(set) = set else {
        return Decision::deny(
            RuleClass::Radius,
            None,
            Some(posix.as_posix()),
            "no working set; run leash session",
        );
    };
    if !set.contains_posix(posix.as_posix(), volume_is_case_insensitive()) {
        return Decision::deny(
            RuleClass::Radius,
            None,
            Some(posix.as_posix()),
            "path is outside the working set",
        );
    }

    let added = if input.tool_name == "Edit" {
        input
            .tool_input
            .get("new_string")
            .and_then(Value::as_str)
            .unwrap_or("")
    } else {
        input
            .tool_input
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or("")
    };
    let apply = [posix.as_posix()];
    if let Some(d) = scan_hunk(posix.as_posix(), added, &apply) {
        return d;
    }
    Decision::allow()
}

fn decide_shell(repo: &Path, set: Option<&WorkingSet>, input: &HookInput) -> Decision {
    let cmd = input
        .tool_input
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if cmd.is_empty() || !is_mutating_shell(cmd) {
        return Decision::allow();
    }
    let Some(set) = set else {
        return Decision::deny(
            RuleClass::Radius,
            None,
            None,
            "mutating bash with no working set; run leash session",
        );
    };
    if let Some(paths) = shell_paths(cmd) {
        for raw in paths {
            match canonicalize_in_repo(repo, &raw) {
                Ok(p) => {
                    if !set.contains_posix(p.as_posix(), volume_is_case_insensitive()) {
                        return Decision::deny(
                            RuleClass::Radius,
                            None,
                            Some(p.as_posix()),
                            "mutating bash path is outside the working set",
                        );
                    }
                }
                Err(_) => {
                    return Decision::deny(
                        RuleClass::PathUncanonical,
                        None,
                        None,
                        "mutating bash path is not a canonical repo path",
                    );
                }
            }
        }
        return Decision::allow();
    }
    Decision::deny(
        RuleClass::Radius,
        None,
        None,
        "mutating bash is not in the working set",
    )
}

fn is_mutating_shell(cmd: &str) -> bool {
    let c = cmd.trim_start();
    let lower = c.to_ascii_lowercase();
    starts_cmd(&lower, "rm ")
        || starts_cmd(&lower, "rm\t")
        || starts_cmd(&lower, "mv ")
        || starts_cmd(&lower, "git commit")
        || starts_cmd(&lower, "git push")
        || starts_cmd(&lower, "gh ")
        || starts_cmd(&lower, "npm publish")
        || starts_cmd(&lower, "cargo publish")
        || starts_cmd(&lower, "remove-item")
        || starts_cmd(&lower, "move-item")
        || starts_cmd(&lower, "set-content")
        || starts_cmd(&lower, "out-file")
        || starts_cmd(&lower, "new-item")
        || c.contains(" > ")
        || c.contains(">>")
}

fn starts_cmd(lower: &str, prefix: &str) -> bool {
    lower.starts_with(prefix) || lower.contains(&format!("&& {prefix}")) || lower.contains(&format!("; {prefix}"))
}

fn shell_paths(cmd: &str) -> Option<Vec<String>> {
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("git commit")
        || lower.contains("git push")
        || lower.contains("npm publish")
        || lower.contains("cargo publish")
        || lower.starts_with("gh ")
    {
        return None;
    }
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut paths = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if *p == "rm" || *p == "mv" || p.eq_ignore_ascii_case("Remove-Item") || p.eq_ignore_ascii_case("Move-Item")
        {
            for later in &parts[i + 1..] {
                if later.starts_with('-') {
                    continue;
                }
                paths.push((*later).trim_matches('"').to_string());
            }
            break;
        }
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

pub fn to_claude_json(decision: &Decision) -> Value {
    if decision.allow {
        json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow"
            }
        })
    } else {
        json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": decision.one_line()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SetPath, WorkingSet};
    use std::fs;

    fn repo_with(set_paths: &[&str]) -> (tempfile::TempDir, WorkingSet) {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src").join("lib.rs"), "ok").unwrap();
        let mut set = WorkingSet::empty("t", 8000);
        for p in set_paths {
            set.paths.push(SetPath {
                path: (*p).into(),
                reason: "changed".into(),
                symbols: vec![],
            });
        }
        (tmp, set)
    }

    fn write_input(path: &str, content: &str) -> HookInput {
        HookInput {
            hook_event_name: "PreToolUse".into(),
            tool_name: "Write".into(),
            tool_input: json!({ "file_path": path, "content": content }),
            cwd: None,
        }
    }

    #[test]
    fn out_of_set_write_is_radius() {
        let (tmp, set) = repo_with(&["src/lib.rs"]);
        let abs = tmp.path().join("src").join("other.rs");
        let d = decide(tmp.path(), Some(&set), &write_input(abs.to_str().unwrap(), "x"));
        assert!(!d.allow);
        assert_eq!(d.class, Some(RuleClass::Radius));
    }

    #[test]
    fn in_set_write_is_allow() {
        let (tmp, set) = repo_with(&["src/lib.rs"]);
        let abs = tmp.path().join("src").join("lib.rs");
        let d = decide(tmp.path(), Some(&set), &write_input(abs.to_str().unwrap(), "fn x() {}"));
        assert!(d.allow);
    }

    #[test]
    fn in_set_secret_is_rule() {
        let (tmp, set) = repo_with(&["src/lib.rs"]);
        let abs = tmp.path().join("src").join("lib.rs");
        let d = decide(
            tmp.path(),
            Some(&set),
            &write_input(abs.to_str().unwrap(), "const api_key = \"sk-abcdefghijklmnopqrstuvwxyz\";"),
        );
        assert_eq!(d.rule_id.as_deref(), Some("LEASH-SEC-001"));
    }

    #[test]
    fn missing_session_denies_write() {
        let (tmp, _) = repo_with(&["src/lib.rs"]);
        let abs = tmp.path().join("src").join("lib.rs");
        let d = decide(tmp.path(), None, &write_input(abs.to_str().unwrap(), "x"));
        assert!(!d.allow);
        assert_eq!(d.class, Some(RuleClass::Radius));
    }

    #[test]
    fn read_only_bash_is_allow() {
        let (tmp, set) = repo_with(&["src/lib.rs"]);
        let input = HookInput {
            hook_event_name: "PreToolUse".into(),
            tool_name: "Bash".into(),
            tool_input: json!({ "command": "git status" }),
            cwd: None,
        };
        assert!(decide(tmp.path(), Some(&set), &input).allow);
    }

    #[test]
    fn git_push_is_denied() {
        let (tmp, set) = repo_with(&["src/lib.rs"]);
        let input = HookInput {
            hook_event_name: "PreToolUse".into(),
            tool_name: "Bash".into(),
            tool_input: json!({ "command": "git push origin main" }),
            cwd: None,
        };
        assert!(!decide(tmp.path(), Some(&set), &input).allow);
    }

    #[test]
    fn deny_json_uses_hook_specific_output() {
        let d = Decision::deny(RuleClass::Radius, None, Some("a.rs"), "outside");
        let v = to_claude_json(&d);
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    }
}
