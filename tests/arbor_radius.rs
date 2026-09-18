//! Arbor fixture JSON packs into a ranked working set without spawning Arbor.

use leash::arbor::{parse_named_files, radius_from_diff, Neighbor};
use leash::config::Config;
use leash::engine::assemble;
use std::fs;
use std::path::Path;

fn load(name: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(Path::new("fixtures/arbor").join(name)).unwrap()).unwrap()
}

#[test]
fn fixture_diff_json_is_arbor_engine() {
    let tmp = tempfile::tempdir().unwrap();
    for (rel, body) in [
        ("src/auth.rs", "login"),
        ("src/crypto.rs", "hash"),
        ("src/http.rs", "handle"),
        ("tests/auth_test.rs", "test"),
    ] {
        let p = tmp.path().join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }
    let mut radius = radius_from_diff(&load("diff.json"));
    for (name, path) in parse_named_files(&load("file-graph.json")) {
        if !name.is_empty() {
            radius.changed_symbols.push((name, path));
        }
    }
    for (name, path) in parse_named_files(&load("callees.json")) {
        radius.callees.push(Neighbor {
            path,
            symbols: vec![name],
        });
    }
    for (name, path) in parse_named_files(&load("callers.json")) {
        radius.callers.push(Neighbor {
            path,
            symbols: vec![name],
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
    assert!(set.paths.iter().any(|p| p.reason == "changed"));
    assert!(set.paths.iter().any(|p| p.reason == "callee"));
    assert!(set.paths.iter().any(|p| p.reason == "caller"));
    assert!(set.paths.iter().any(|p| p.reason == "test"));
}
