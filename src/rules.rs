//! Agent-failure ruleset. Cheap, named, fixture-backed.

use crate::decision::{Decision, RuleClass};

pub fn scan_added(path: &str, added: &str) -> Option<Decision> {
    scan_hunk(path, added, &[])
}

pub fn scan_hunk(path: &str, added: &str, apply_paths: &[&str]) -> Option<Decision> {
    let _ = apply_paths;
    sec_001(path, added)
        .or_else(|| sec_002(path, added))
        .or_else(|| sec_003(path, added))
        .or_else(|| sec_004(path, added))
}

fn added_lines(added: &str) -> impl Iterator<Item = (usize, &str)> {
    added.lines().enumerate().map(|(i, l)| (i + 1, l))
}

fn sec_002(path: &str, added: &str) -> Option<Decision> {
    for (line_no, line) in added_lines(added) {
        if dynamic_eval(line) {
            return Some(Decision::deny(
                RuleClass::Rule,
                Some("LEASH-SEC-002"),
                Some(path),
                &format!("eval/exec on non-literal input in added lines (line {line_no})"),
            ));
        }
    }
    None
}

fn dynamic_eval(line: &str) -> bool {
    let t = line.trim();
    // Literal-only calls are not the agent footgun this rule is for.
    let calls = ["eval(", "exec(", "Function("];
    for c in calls {
        if let Some(idx) = t.find(c) {
            let rest = t[idx + c.len()..].trim_start();
            if rest.starts_with('"') || rest.starts_with('\'') || rest.starts_with('`') {
                continue;
            }
            if rest.starts_with(')') {
                continue;
            }
            return true;
        }
    }
    false
}

fn sec_003(path: &str, added: &str) -> Option<Decision> {
    for (line_no, line) in added_lines(added) {
        if sql_concat(line) {
            return Some(Decision::deny(
                RuleClass::Rule,
                Some("LEASH-SEC-003"),
                Some(path),
                &format!("SQL string concat in added lines (line {line_no})"),
            ));
        }
    }
    None
}

fn sql_concat(line: &str) -> bool {
    let u = line.to_ascii_uppercase();
    let sql = u.contains("SELECT ")
        || u.contains("INSERT ")
        || u.contains("UPDATE ")
        || u.contains("DELETE ");
    if !sql {
        return false;
    }
    line.contains(" + ")
        || line.contains("${")
        || line.contains("%s")
        || line.contains(".format(")
        || (line.contains("f\"") && line.contains('{'))
        || (line.contains("f'") && line.contains('{'))
}

fn sec_004(path: &str, added: &str) -> Option<Decision> {
    for (line_no, line) in added_lines(added) {
        if xss_sink(line) {
            return Some(Decision::deny(
                RuleClass::Rule,
                Some("LEASH-SEC-004"),
                Some(path),
                &format!("HTML interpolated into a DOM sink in added lines (line {line_no})"),
            ));
        }
    }
    None
}

fn xss_sink(line: &str) -> bool {
    let sink = line.contains("innerHTML")
        || line.contains("outerHTML")
        || line.contains("document.write")
        || line.contains("dangerouslySetInnerHTML")
        || line.contains("insertAdjacentHTML");
    if !sink {
        return false;
    }
    line.contains("${")
        || line.contains(" + ")
        || line.contains("{") && line.contains("}")
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
    fn eval_on_variable_denies() {
        let d = scan_added("app.js", "eval(userInput);\n").unwrap();
        assert_eq!(d.rule_id.as_deref(), Some("LEASH-SEC-002"));
    }

    #[test]
    fn eval_on_literal_is_clean() {
        assert!(scan_added("app.js", "eval(\"2+2\");\n").is_none());
    }

    #[test]
    fn sql_concat_denies() {
        let d = scan_added("db.py", "q = \"SELECT * FROM users WHERE id = \" + user_id\n").unwrap();
        assert_eq!(d.rule_id.as_deref(), Some("LEASH-SEC-003"));
    }

    #[test]
    fn parameterized_sql_is_clean() {
        assert!(scan_added("db.py", "q = \"SELECT * FROM users WHERE id = ?\"\n").is_none());
    }

    #[test]
    fn innerhtml_interpolation_denies() {
        let d = scan_added("ui.js", "el.innerHTML = `<div>${user}</div>`;\n").unwrap();
        assert_eq!(d.rule_id.as_deref(), Some("LEASH-SEC-004"));
    }

    #[test]
    fn innerhtml_literal_is_clean() {
        assert!(scan_added("ui.js", "el.innerHTML = \"<div class='ok'></div>\";\n").is_none());
    }
}
