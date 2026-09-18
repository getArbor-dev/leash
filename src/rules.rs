//! Agent-failure ruleset. Cheap, named, fixture-backed.

use crate::decision::{Decision, RuleClass};

pub fn scan_added(path: &str, added: &str) -> Option<Decision> {
    sec_001(path, added)
}

fn added_lines(added: &str) -> impl Iterator<Item = (usize, &str)> {
    added.lines().enumerate().map(|(i, l)| (i + 1, l))
}

fn sec_001(path: &str, added: &str) -> Option<Decision> {
    for (line_no, line) in added_lines(added) {
        if secret_shaped(line) {
            return Some(Decision::deny(
                RuleClass::Rule,
                Some("LEASH-SEC-001"),
                Some(path),
                &format!("secret-shaped assignment in added lines (line {line_no}); rotate if this was real"),
            ));
        }
    }
    None
}

fn secret_shaped(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let looks_assign = lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("private_key")
        || lower.contains("access_token");
    if looks_assign {
        if quoted_value_len(line) >= 12 {
            return true;
        }
    }
    contains_token_prefix(line)
}

fn quoted_value_len(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let q = bytes[i];
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != q {
                i += 1;
            }
            return i.saturating_sub(start);
        }
        i += 1;
    }
    0
}

fn contains_token_prefix(line: &str) -> bool {
    prefixed_token(line, "AKIA", 16)
        || prefixed_token(line, "ghp_", 20)
        || prefixed_token(line, "sk-", 20)
        || prefixed_token(line, "xoxb-", 20)
        || prefixed_token(line, "xoxp-", 20)
}

fn prefixed_token(line: &str, prefix: &str, min_rest: usize) -> bool {
    let Some(idx) = line.find(prefix) else {
        return false;
    };
    let rest = &line[idx + prefix.len()..];
    let n = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .count();
    n >= min_rest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_assignment_denies_without_echoing() {
        let d = scan_added(
            "src/config.ts",
            "const api_key = \"sk-abcdefghijklmnopqrstuvwxyz\";\n",
        )
        .unwrap();
        assert!(!d.allow);
        assert_eq!(d.rule_id.as_deref(), Some("LEASH-SEC-001"));
        let line = d.one_line();
        assert!(!line.contains("sk-abcdefghijklmnopqrstuvwxyz"));
    }

    #[test]
    fn ordinary_code_is_clean() {
        assert!(scan_added("src/lib.rs", "let name = \"leash\";\n").is_none());
    }
}
