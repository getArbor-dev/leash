//! argv parsing. No extra CLI crate: the binary has to start in a hook.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    Session { task: String, paths: Vec<String> },
    SessionFromHook,
    Hook,
    Status,
    Expand { path: String, reason: String },
    Install,
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
            let mut path = String::new();
            let mut reason = String::new();
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        i += 1;
                        path = rest.get(i).cloned().ok_or("--path needs a value")?;
                    }
                    "--reason" => {
                        i += 1;
                        reason = rest.get(i).cloned().ok_or("--reason needs a value")?;
                    }
                    other => return Err(format!("unknown expand flag {other}")),
                }
                i += 1;
            }
            if path.is_empty() || reason.is_empty() {
                return Err("expand requires --path and --reason".into());
            }
            Ok(Cmd::Expand { path, reason })
        }
        other => Err(format!("unknown command {other}")),
    }
}

pub fn help_text() -> &'static str {
    "leash session --task TEXT [--path PATH]...\n\
     leash hook\n\
     leash status\n\
     leash expand --path PATH --reason TEXT\n\
     leash install\n"
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
}
