//! CI door: same pack and ruleset, no session file, no GitHub client.

use std::path::Path;

use crate::config;
use crate::decision::Decision;
use crate::diff::{self, DiffError};
use crate::engine;
use crate::paths::volume_is_case_insensitive;
use crate::rules::scan_hunk;
use crate::session::WorkingSet;

#[derive(Debug)]
pub struct Report {
    pub set: WorkingSet,
    pub denials: Vec<Decision>,
    pub range: String,
    pub omitted_paths: Vec<String>,
}

pub fn range_for_base(base: &str) -> Result<String, DiffError> {
    diff::validate_rev(base)?;
    Ok(format!("{base}...HEAD"))
}

pub fn run(repo: &Path, base: &str, task: &str) -> Result<Report, DiffError> {
    let range = range_for_base(base)?;
    let git_changed = diff::changed_against(repo, &range)?;
    let cfg = config::load(repo);
    let set = engine::assemble(
        repo,
        task,
        &[],
        &cfg,
        git_changed.clone(),
        None,
        vec![
            "engine=diff".into(),
            "door=ci".into(),
            format!("base={base}"),
        ],
    );
    let added = diff::added_by_path(repo, &range)?;
    let apply: Vec<&str> = git_changed.iter().map(String::as_str).collect();
    let mut denials = Vec::new();
    for (path, body) in &added {
        if let Some(d) = scan_hunk(path, body, &apply) {
            denials.push(d);
        }
    }
    let omitted_paths = omitted_from_range(&set, &git_changed);
    Ok(Report {
        set,
        denials,
        range,
        omitted_paths,
    })
}

fn omitted_from_range(set: &WorkingSet, git_changed: &[String]) -> Vec<String> {
    let casefold = volume_is_case_insensitive();
    git_changed
        .iter()
        .filter(|p| !set.contains_posix(&p.replace('\\', "/"), casefold))
        .cloned()
        .collect()
}

pub fn exit_code(report: &Report) -> i32 {
    if report.denials.is_empty() {
        0
    } else {
        1
    }
}

/// Working set + denials. Not a review essay. No file bodies.
pub fn markdown(report: &Report) -> String {
    let mut out = String::from("<!-- leash -->\n## Leash\n\n");
    out.push_str(&format!(
        "engine: `{}`  \nrange: `{}`  \nbudget: {}/{}\n\n",
        report.set.engine, report.range, report.set.used_tokens, report.set.budget_tokens
    ));
    out.push_str("### Working set\n\n");
    if report.set.paths.is_empty() {
        out.push_str("_empty — no paths in range_\n\n");
    } else {
        for p in &report.set.paths {
            out.push_str(&format!("- `{}` ({})\n", p.path, p.reason));
        }
        out.push('\n');
    }
    if !report.omitted_paths.is_empty() || !report.set.omitted.is_empty() {
        out.push_str("### Omitted\n\n");
        for p in &report.omitted_paths {
            out.push_str(&format!("- `{p}`\n"));
        }
        for o in &report.set.omitted {
            out.push_str(&format!("- {}: {}\n", o.reason, o.count));
        }
        out.push('\n');
    }
    out.push_str("### Denials\n\n");
    if report.denials.is_empty() {
        out.push_str("_none_\n");
    } else {
        for d in &report.denials {
            out.push_str(&format!("```\n{}\n```\n", d.one_line()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::parse_unified_added;
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
        Command::new("git")
            .args(["checkout", "-b", "main"])
            .current_dir(p)
            .status()
            .ok();
        fs::write(p.join("keep.txt"), "keep\n").unwrap();
        assert!(Command::new("git").args(["add", "keep.txt"]).current_dir(p).status().unwrap().success());
        assert!(Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "init"])
            .current_dir(p)
            .status()
            .unwrap()
            .success());
        tmp
    }

    fn commit_all(p: &Path, msg: &str) {
        Command::new("git").args(["add", "-A"]).current_dir(p).status().unwrap();
        Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", msg])
            .current_dir(p)
            .status()
            .unwrap();
    }

    #[test]
    fn parse_skips_deletes_and_headers() {
        let diff = "\
diff --git a/gone.txt b/gone.txt
--- a/gone.txt
+++ /dev/null
@@ -1 +0,0 @@
-old
diff --git a/keep.txt b/keep.txt
--- a/keep.txt
+++ b/keep.txt\t2026-01-01 00:00:00
@@ -1 +1 @@
-keep
+changed
";
        let got = parse_unified_added(diff);
        assert_eq!(got, vec![("keep.txt".into(), "changed".into())]);
    }

    fn root_commit(p: &Path) -> String {
        let out = Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(p)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    fn branch_off(p: &Path, name: &str) {
        assert!(Command::new("git")
            .args(["checkout", "-b", name])
            .current_dir(p)
            .status()
            .unwrap()
            .success());
    }

    #[test]
    fn clean_range_is_exit_zero() {
        let tmp = git_repo();
        let p = tmp.path();
        branch_off(p, "pr");
        fs::write(p.join("keep.txt"), "changed\n").unwrap();
        commit_all(p, "edit");
        let report = run(p, "HEAD~1", "pull request").unwrap();
        assert_eq!(exit_code(&report), 0);
        assert!(report.set.contains_posix("keep.txt", cfg!(windows)));
        let md = markdown(&report);
        assert!(md.contains("<!-- leash -->"));
        assert!(md.contains("_none_"));
        assert!(md.contains("keep.txt"));
    }

    #[test]
    fn ruleset_hunk_is_exit_one() {
        let tmp = git_repo();
        let p = tmp.path();
        // Split so this test file's own PR hunk does not trip the Action.
        let body = format!("{}({});\n", "eval", "userInput");
        fs::write(p.join("app.js"), body).unwrap();
        commit_all(p, "footgun");
        let report = run(p, &root_commit(p), "pull request").unwrap();
        assert_eq!(exit_code(&report), 1);
        assert_eq!(
            report.denials[0].rule_id.as_deref(),
            Some("LEASH-SEC-002")
        );
        let md = markdown(&report);
        assert!(md.contains("LEASH-SEC-002"));
        assert!(!md.contains(&format!("{}({})", "eval", "userInput")));
    }

    #[test]
    fn lockfile_in_range_clears_manifest_rule() {
        let tmp = git_repo();
        let p = tmp.path();
        fs::write(
            p.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(p.join("Cargo.lock"), "# lock\n").unwrap();
        commit_all(p, "crate");
        fs::write(
            p.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nserde = \"1\"\n",
        )
        .unwrap();
        fs::write(p.join("Cargo.lock"), "# lock\n# more\n").unwrap();
        commit_all(p, "dep");
        let report = run(p, &root_commit(p), "pull request").unwrap();
        assert!(
            report.denials.is_empty(),
            "unexpected denials: {:?}",
            report.denials
        );
    }

    #[test]
    fn over_budget_comment_names_omitted_paths() {
        let tmp = git_repo();
        let p = tmp.path();
        branch_off(p, "pr");
        fs::write(p.join("leash.yml"), "budget_tokens: 4\n").unwrap();
        fs::write(p.join("big-a.txt"), "a".repeat(80)).unwrap();
        fs::write(p.join("big-b.txt"), "b".repeat(80)).unwrap();
        commit_all(p, "two files");
        let report = run(p, "HEAD~1", "pull request").unwrap();
        assert_eq!(exit_code(&report), 0);
        assert!(!report.omitted_paths.is_empty());
        let md = markdown(&report);
        assert!(
            report.omitted_paths.iter().any(|p| md.contains(&format!("`{p}`"))),
            "comment should name omitted paths: {md}"
        );
    }

    #[test]
    fn rejects_flag_shaped_base() {
        assert!(range_for_base("--output").is_err());
        assert!(range_for_base("main HEAD").is_err());
    }
}
