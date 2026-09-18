//! `.leash/session.json` load and save. Directory is gitignored.

use std::fs;
use std::path::{Path, PathBuf};

use crate::session::WorkingSet;

pub fn session_path(repo: &Path) -> PathBuf {
    repo.join(".leash").join("session.json")
}

pub fn load(repo: &Path) -> Option<WorkingSet> {
    let text = fs::read_to_string(session_path(repo)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(repo: &Path, set: &WorkingSet) -> std::io::Result<()> {
    let dir = repo.join(".leash");
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(set).map_err(std::io::Error::other)?;
    fs::write(session_path(repo), json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let set = WorkingSet::empty("t", 8000);
        save(tmp.path(), &set).unwrap();
        let back = load(tmp.path()).unwrap();
        assert_eq!(back.task, "t");
        assert_eq!(back.engine, "diff");
    }
}
