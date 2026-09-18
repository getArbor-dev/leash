//! Optional `leash.yml`. Missing file is the documented default, not an error.

use std::path::Path;

use crate::session::DEFAULT_BUDGET_TOKENS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub budget_tokens: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            budget_tokens: DEFAULT_BUDGET_TOKENS,
        }
    }
}

pub fn load(repo: &Path) -> Config {
    let path = repo.join("leash.yml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Config::default();
    };
    parse(&text)
}

pub fn parse(text: &str) -> Config {
    let mut cfg = Config::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        if k.trim() == "budget_tokens" {
            if let Ok(n) = v.trim().parse::<u32>() {
                cfg.budget_tokens = n;
            }
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_default() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(load(tmp.path()), Config::default());
    }

    #[test]
    fn reads_budget() {
        assert_eq!(parse("budget_tokens: 1200\n").budget_tokens, 1200);
    }
}
