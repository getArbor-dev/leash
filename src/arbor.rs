//! Arbor v3 subprocess. Never claim a graph walk that did not happen.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

pub const TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_SYMBOLS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArborError {
    Missing,
    Failed(String),
    Timeout,
    Unparseable,
}

impl ArborError {
    pub fn audit_tag(&self) -> &'static str {
        match self {
            ArborError::Missing => "arbor_missing",
            ArborError::Timeout => "arbor_timeout",
            ArborError::Failed(_) | ArborError::Unparseable => "arbor_failed",
        }
    }
}

impl std::fmt::Display for ArborError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArborError::Missing => write!(f, "arbor not found on PATH"),
            ArborError::Failed(s) => write!(f, "arbor failed: {s}"),
            ArborError::Timeout => write!(f, "arbor timed out"),
            ArborError::Unparseable => write!(f, "arbor JSON could not be parsed"),
        }
    }
}

impl std::error::Error for ArborError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Auto,
    Bin(PathBuf),
    Off,
}

impl Probe {
    pub fn from_env() -> Self {
        match std::env::var("ARBOR_BIN") {
            Ok(p) if !p.is_empty() => Probe::Bin(PathBuf::from(p)),
            _ => Probe::Auto,
        }
    }

    fn command(&self) -> Result<Command, ArborError> {
        match self {
            Probe::Off => Err(ArborError::Missing),
            Probe::Auto => Ok(Command::new("arbor")),
            Probe::Bin(p) => Ok(Command::new(p)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Neighbor {
    pub path: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArborRadius {
    pub changed_files: Vec<String>,
    pub changed_symbols: Vec<(String, String)>,
    pub callees: Vec<Neighbor>,
    pub callers: Vec<Neighbor>,
}

pub fn collect(repo: &Path, probe: &Probe, timeout: Duration) -> Result<ArborRadius, ArborError> {
    let diff_raw = run_json(probe, repo, &["diff", ".", "--json"], timeout)?;
    let diff = serde_json::from_str::<Value>(&diff_raw).map_err(|_| ArborError::Unparseable)?;
    let mut radius = radius_from_diff(&diff);

    if radius.changed_symbols.is_empty() {
        let files: Vec<String> = radius.changed_files.iter().take(MAX_SYMBOLS).cloned().collect();
        for file in files {
            match run_json(probe, repo, &["file-graph", &file, ".", "--json"], timeout) {
                Ok(raw) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                        for (name, path) in parse_named_files(&v) {
                            push_symbol(&mut radius.changed_symbols, name, path);
                        }
                    }
                }
                Err(ArborError::Missing) => return Err(ArborError::Missing),
                Err(_) => continue,
            }
        }
    }

    let symbols: Vec<(String, String)> = radius
        .changed_symbols
        .iter()
        .take(MAX_SYMBOLS)
        .cloned()
        .collect();
    for (name, _) in symbols {
        if let Ok(raw) = run_json(probe, repo, &["callees", &name, ".", "--json"], timeout) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                merge_neighbors(&mut radius.callees, parse_named_files(&v));
            }
        }
        if let Ok(raw) = run_json(probe, repo, &["callers", &name, ".", "--json"], timeout) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                merge_neighbors(&mut radius.callers, parse_named_files(&v));
            }
        }
    }
    Ok(radius)
}

pub fn radius_from_diff(diff: &Value) -> ArborRadius {
    let mut radius = ArborRadius::default();
    radius.changed_files = string_list(diff, &["changed_files", "files"]);
    radius.changed_symbols = parse_named_files(diff)
        .into_iter()
        .filter(|(n, _)| !n.is_empty())
        .collect();
    if let Some(impact) = diff.get("impact") {
        merge_neighbors(&mut radius.callers, parse_named_files(impact));
        if let Some(arr) = impact.get("direct_callers") {
            if arr.is_array() {
                merge_neighbors(&mut radius.callers, parse_named_files(arr));
            }
        }
        if let Some(arr) = impact.get("direct_callees") {
            if arr.is_array() {
                merge_neighbors(&mut radius.callees, parse_named_files(arr));
            }
        }
    }
    radius
}

pub fn parse_named_files(v: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    walk_named(v, &mut out);
    out
}

fn walk_named(v: &Value, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            let name = m
                .get("name")
                .or_else(|| m.get("symbol"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let file = m
                .get("file")
                .or_else(|| m.get("path"))
                .and_then(Value::as_str)
                .map(|s| s.replace('\\', "/"));
            if let Some(file) = file {
                if looks_like_path(&file) {
                    out.push((name.to_string(), file));
                }
            }
            for val in m.values() {
                walk_named(val, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                if let Some(s) = x.as_str() {
                    if looks_like_path(s) {
                        out.push((String::new(), s.replace('\\', "/")));
                    }
                } else {
                    walk_named(x, out);
                }
            }
        }
        _ => {}
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.contains('.')
}

fn string_list(v: &Value, keys: &[&str]) -> Vec<String> {
    for k in keys {
        if let Some(Value::Array(a)) = v.get(*k) {
            return a
                .iter()
                .filter_map(Value::as_str)
                .map(|s| s.replace('\\', "/"))
                .collect();
        }
    }
    Vec::new()
}

fn push_symbol(out: &mut Vec<(String, String)>, name: String, path: String) {
    if name.is_empty() {
        return;
    }
    if !out.iter().any(|(n, p)| n == &name && p == &path) {
        out.push((name, path));
    }
}

fn merge_neighbors(out: &mut Vec<Neighbor>, hits: Vec<(String, String)>) {
    for (name, path) in hits {
        if path.is_empty() {
            continue;
        }
        if let Some(n) = out.iter_mut().find(|n| n.path == path) {
            if !name.is_empty() && !n.symbols.iter().any(|s| s == &name) {
                n.symbols.push(name);
            }
        } else {
            let symbols = if name.is_empty() { vec![] } else { vec![name] };
            out.push(Neighbor { path, symbols });
        }
    }
}

fn run_json(probe: &Probe, repo: &Path, args: &[&str], timeout: Duration) -> Result<String, ArborError> {
    use std::io::Read;
    let mut cmd = probe.command()?;
    cmd.args(args)
        .current_dir(repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|_| ArborError::Missing)?;
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut out = Vec::new();
                let mut err = Vec::new();
                if let Some(ref mut s) = stdout {
                    let _ = s.read_to_end(&mut out);
                }
                if let Some(ref mut s) = stderr {
                    let _ = s.read_to_end(&mut err);
                }
                if !status.success() {
                    return Err(ArborError::Failed(
                        String::from_utf8_lossy(&err).trim().to_string(),
                    ));
                }
                return String::from_utf8(out).map_err(|_| ArborError::Unparseable);
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ArborError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Err(e) => return Err(ArborError::Failed(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn load(name: &str) -> Value {
        let text = std::fs::read_to_string(format!("fixtures/arbor/{name}")).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn diff_fixture_lists_changed_files() {
        let r = radius_from_diff(&load("diff.json"));
        assert_eq!(r.changed_files, vec!["src/auth.rs"]);
    }

    #[test]
    fn file_graph_and_neighbors_parse() {
        let symbols = parse_named_files(&load("file-graph.json"));
        assert!(symbols.iter().any(|(n, p)| n == "login" && p == "src/auth.rs"));
        let callees = parse_named_files(&load("callees.json"));
        assert!(callees.iter().any(|(n, p)| n == "hash_password" && p == "src/crypto.rs"));
        let callers = parse_named_files(&load("callers.json"));
        assert!(callers.iter().any(|(n, p)| n == "handle_login" && p == "src/http.rs"));
        assert!(callers.iter().any(|(_, p)| p == "tests/auth_test.rs"));
    }

    #[test]
    fn missing_binary_is_arbor_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let err = collect(
            tmp.path(),
            &Probe::Bin(PathBuf::from("__leash_no_arbor__")),
            Duration::from_millis(200),
        )
        .unwrap_err();
        assert_eq!(err, ArborError::Missing);
        assert_eq!(err.audit_tag(), "arbor_missing");
    }

    #[test]
    fn off_probe_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let err = collect(tmp.path(), &Probe::Off, Duration::from_millis(50)).unwrap_err();
        assert_eq!(err.audit_tag(), "arbor_missing");
    }
}
