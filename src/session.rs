//! Working set on disk: `.leash/session.json`.

use serde::{Deserialize, Serialize};

pub const DEFAULT_BUDGET_TOKENS: u32 = 8000;
pub const TOKENIZER_APPROX: &str = "approx-chars-div-4";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingSet {
    pub task: String,
    pub budget_tokens: u32,
    pub used_tokens: u32,
    pub tokenizer: String,
    /// `diff` until Arbor is wired. Never claim `arbor` without running it.
    pub engine: String,
    pub paths: Vec<SetPath>,
    pub omitted: Vec<Omitted>,
    pub denials: Vec<DenialRecord>,
    pub audit: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetPath {
    pub path: String,
    pub reason: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Omitted {
    pub reason: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenialRecord {
    pub rule: String,
    pub path: String,
    pub action: String,
}

impl WorkingSet {
    pub fn empty(task: &str, budget_tokens: u32) -> Self {
        Self {
            task: task.to_string(),
            budget_tokens,
            used_tokens: 0,
            tokenizer: TOKENIZER_APPROX.to_string(),
            engine: "diff".to_string(),
            paths: Vec::new(),
            omitted: Vec::new(),
            denials: Vec::new(),
            audit: Vec::new(),
        }
    }

    pub fn contains_posix(&self, posix: &str, case_insensitive: bool) -> bool {
        self.paths.iter().any(|p| {
            if case_insensitive {
                p.path.eq_ignore_ascii_case(posix)
            } else {
                p.path == posix
            }
        })
    }

    pub fn contains_symbol(&self, symbol: &str) -> bool {
        self.paths
            .iter()
            .any(|p| p.symbols.iter().any(|s| s.eq_ignore_ascii_case(symbol)))
    }

    pub fn approx_tokens(bytes: usize) -> u32 {
        (bytes as u32).div_ceil(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip_keeps_engine_label() {
        let mut set = WorkingSet::empty("fix hook", DEFAULT_BUDGET_TOKENS);
        set.paths.push(SetPath {
            path: "src/hook.rs".into(),
            reason: "changed".into(),
            symbols: vec![],
        });
        let json = serde_json::to_string(&set).unwrap();
        let back: WorkingSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.engine, "diff");
        assert!(back.contains_posix("src/hook.rs", false));
    }

    #[test]
    fn approx_tokens_is_chars_div_4() {
        assert_eq!(WorkingSet::approx_tokens(0), 0);
        assert_eq!(WorkingSet::approx_tokens(4), 1);
        assert_eq!(WorkingSet::approx_tokens(5), 2);
    }
}
