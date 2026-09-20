//! argv parsing. No extra CLI crate: the binary has to start in a hook.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    Session { task: String, paths: Vec<String> },
    SessionFromHook,
    Hook,
    Status,
    Expand { path: Option<String>, symbol: Option<String>, reason: String },
    Install,
    Ci { base: String, task: String },
    Help,
}

pub fn parse(args: &[String]) -> Result<Cmd, String> {
    let mut it = args.iter();
    let Some(cmd) = it.next() else {
        return Ok(Cmd::Help);
    };
    match cmd.as_str() {
        "-h" | "--help" | "help" => Ok(Cmd::Help),
        "hook" => Ok(Cmd::Hook),
        "status" => Ok(Cmd::Status),
        "install" => Ok(Cmd::Install),
        "ci" => {
            let rest: Vec<String> = it.cloned().collect();
            let mut base = String::new();
            let mut task = String::from("pull request");
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--base" => {
                        i += 1;
                        base = rest.get(i).cloned().ok_or("--base needs a value")?;
                    }
                    "--task" => {
                        i += 1;
                        task = rest.get(i).cloned().ok_or("--task needs a value")?;
                    }
                    other if other.starts_with("--base=") => {
                        base = other[7..].to_string();
                    }
                    other if other.starts_with("--task=") => {
                        task = other[7..].to_string();
                    }
                    other => return Err(format!("unknown ci flag {other}")),
                }
                i += 1;
            }
            if base.is_empty() {
                if let Ok(env_base) = std::env::var("LEASH_BASE") {
                    if !env_base.is_empty() {
                        base = env_base;
                    }
                }
            }
            if base.is_empty() {
                return Err("ci requires --base".into());
            }
            Ok(Cmd::Ci { base, task })
        }
        "session" => {
            let rest: Vec<String> = it.cloned().collect();
            if rest.iter().any(|a| a == "--from-hook") {
                return Ok(Cmd::SessionFromHook);
            }
            let mut task = String::new();
            let mut paths = Vec::new();
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--task" => {
                        i += 1;
                        task = rest.get(i).cloned().ok_or("--task needs a value")?;
                    }
                    "--path" => {
                        i += 1;
                        paths.push(rest.get(i).cloned().ok_or("--path needs a value")?);
                    }
                    other if other.starts_with("--task=") => {
                        task = other[7..].to_string();
                    }
                    other => return Err(format!("unknown session flag {other}")),
                }
                i += 1;
            }
            if task.is_empty() {
                return Err("session requires --task".into());
            }
            Ok(Cmd::Session { task, paths })
        }
        "expand" => {
            let rest: Vec<String> = it.cloned().collect();
            let mut path = None;
            let mut symbol = None;
            let mut reason = String::new();
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        i += 1;
                        path = Some(rest.get(i).cloned().ok_or("--path needs a value")?);
                    }
                    "--symbol" => {
                        i += 1;
                        symbol = Some(rest.get(i).cloned().ok_or("--symbol needs a value")?);
                    }
                    "--reason" | "--because" => {
                        i += 1;
                        reason = rest.get(i).cloned().ok_or("expand needs a reason")?;
                    }
                    other => return Err(format!("unknown expand flag {other}")),
                }
                i += 1;
            }
            if path.is_none() && symbol.is_none() {
                return Err("expand requires --path or --symbol".into());
            }
            if reason.is_empty() {
                return Err("expand requires --reason or --because".into());
            }
            Ok(Cmd::Expand {
                path,
                symbol,
                reason,
            })
        }
        other => Err(format!("unknown command {other}")),
    }
}

pub fn help_text() -> &'static str {
    "leash session --task TEXT [--path PATH]...\n\
     leash hook\n\
     leash status\n\
     leash expand --path PATH --reason TEXT\n\
     leash expand --symbol SYM --because TEXT\n\
     leash install\n\
     leash ci --base REF [--task TEXT]\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| (*a).to_string()).collect()
    }

    #[test]
    fn session_task_and_path() {
        let c = parse(&s(&["session", "--task", "fix hook", "--path", "src/a.rs"])).unwrap();
        assert_eq!(
            c,
            Cmd::Session {
                task: "fix hook".into(),
                paths: vec!["src/a.rs".into()],
            }
        );
    }

    #[test]
    fn session_requires_task() {
        assert!(parse(&s(&["session"])).is_err());
    }

    #[test]
    fn expand_requires_both() {
        assert!(parse(&s(&["expand", "--path", "a.rs"])).is_err());
    }

    #[test]
    fn ci_requires_base() {
        assert!(parse(&s(&["ci"])).is_err());
    }

    #[test]
    fn ci_parses_base_and_task() {
        let c = parse(&s(&["ci", "--base", "origin/main", "--task", "pr"])).unwrap();
        assert_eq!(
            c,
            Cmd::Ci {
                base: "origin/main".into(),
                task: "pr".into(),
            }
        );
    }

    #[test]
    fn expand_accepts_because_and_symbol() {
        let c = parse(&s(&[
            "expand",
            "--symbol",
            "login",
            "--because",
            "trace callers",
        ]))
        .unwrap();
        assert_eq!(
            c,
            Cmd::Expand {
                path: None,
                symbol: Some("login".into()),
                reason: "trace callers".into(),
            }
        );
    }
}
