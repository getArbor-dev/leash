//! Fixture pack from the v0 contract: radius, secrets, manifest.

use leash::decision::RuleClass;
use leash::hook::{decide, HookInput};
use leash::session::{SetPath, WorkingSet};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Fixture {
    tool_name: String,
    set_paths: Vec<String>,
    file_rel: String,
    content: String,
    expect_class: Option<String>,
    expect_rule: Option<String>,
}

fn load(path: &Path) -> Fixture {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn run(fx: &Fixture) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    for p in fx.set_paths.iter().chain(std::iter::once(&fx.file_rel)) {
        let abs = repo.join(p.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        if !abs.exists() {
            fs::write(&abs, "").unwrap();
        }
    }
    let mut set = WorkingSet::empty("fixture", 8000);
    for p in &fx.set_paths {
        set.paths.push(SetPath {
            path: p.clone(),
            reason: "changed".into(),
            symbols: vec![],
        });
    }
    let abs = repo.join(fx.file_rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    let input = HookInput {
        hook_event_name: "PreToolUse".into(),
        tool_name: fx.tool_name.clone(),
        tool_input: json!({
            "file_path": abs.to_str().unwrap(),
            "content": fx.content,
        }),
        cwd: Some(repo.to_string_lossy().into()),
    };
    let d = decide(repo, Some(&set), &input);
    assert!(!d.allow, "fixture should deny");
    if let Some(class) = &fx.expect_class {
        assert_eq!(d.class.as_ref().map(RuleClass::as_str).unwrap_or(""), class);
    }
    if let Some(rule) = &fx.expect_rule {
        assert_eq!(d.rule_id.as_deref(), Some(rule.as_str()));
    }
}

#[test]
fn radius() {
    run(&load(Path::new("fixtures/radius.json")));
}

#[test]
fn secrets() {
    run(&load(Path::new("fixtures/secrets.json")));
}

#[test]
fn manifest() {
    run(&load(Path::new("fixtures/manifest.json")));
}
