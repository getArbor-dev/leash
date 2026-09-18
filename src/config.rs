//! Optional `leash.yml`. Missing file is the documented default, not an error.

use std::path::Path;

use crate::session::DEFAULT_BUDGET_TOKENS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub budget_tokens: u32,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            budget_tokens: DEFAULT_BUDGET_TOKENS,
            include: Vec::new(),
            exclude: Vec::new(),
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
    let mut list: Option<&'static str> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(item) = line.strip_prefix("- ") {
            let item = unquote(item.trim());
            if item.is_empty() {
                continue;
            }
            match list {
                Some("include") => cfg.include.push(item),
                Some("exclude") => cfg.exclude.push(item),
                _ => {}
            }
            continue;
        }
        if line == "-" || line == "- []" {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim();
        match k {
            "budget_tokens" => {
                list = None;
                if let Ok(n) = v.parse::<u32>() {
                    cfg.budget_tokens = n;
                }
            }
            "include" => {
                list = Some("include");
                cfg.include.extend(flow_list(v));
                if v == "[]" {
                    list = None;
                }
            }
            "exclude" => {
                list = Some("exclude");
                cfg.exclude.extend(flow_list(v));
                if v == "[]" {
                    list = None;
                }
            }
            _ => list = None,
        }
    }
    cfg
}

fn flow_list(v: &str) -> Vec<String> {
    let v = v.trim();
    if v.is_empty() || v == "[]" {
        return Vec::new();
    }
    let inner = v.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|s| unquote(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn unquote(s: &str) -> String {
    s.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
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

    #[test]
    fn reads_block_globs() {
        let cfg = parse(
            "budget_tokens: 8000\ninclude:\n  - src/**\n  - tests/**\nexclude:\n  - vendor/**\n",
        );
        assert_eq!(cfg.include, vec!["src/**", "tests/**"]);
        assert_eq!(cfg.exclude, vec!["vendor/**"]);
    }

    #[test]
    fn reads_flow_globs() {
        let cfg = parse("include: [src/**, \"lib/**\"]\nexclude: []\n");
        assert_eq!(cfg.include, vec!["src/**", "lib/**"]);
        assert!(cfg.exclude.is_empty());
    }
}
