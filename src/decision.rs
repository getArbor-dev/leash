//! Allow / deny. The hook maps this onto Claude Code JSON.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleClass {
    Radius,
    Rule,
    PathUncanonical,
}

impl RuleClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleClass::Radius => "RADIUS",
            RuleClass::Rule => "RULE",
            RuleClass::PathUncanonical => "PATH_UNCANONICAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub allow: bool,
    pub class: Option<RuleClass>,
    pub rule_id: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

impl Decision {
    pub fn allow() -> Self {
        Self {
            allow: true,
            class: None,
            rule_id: None,
            path: None,
            message: String::new(),
        }
    }

    pub fn deny(class: RuleClass, rule_id: Option<&str>, path: Option<&str>, message: &str) -> Self {
        Self {
            allow: false,
            class: Some(class),
            rule_id: rule_id.map(str::to_string),
            path: path.map(str::to_string),
            message: message.to_string(),
        }
    }

    /// One-line form from the hook contract. Never includes secret bytes.
    pub fn one_line(&self) -> String {
        if self.allow {
            return "allow".to_string();
        }
        let class = self
            .class
            .as_ref()
            .map(RuleClass::as_str)
            .unwrap_or("RULE");
        let mut out = format!("deny {class}");
        if let Some(id) = &self.rule_id {
            out.push(' ');
            out.push_str(id);
        }
        if let Some(path) = &self.path {
            out.push_str(" path=");
            out.push_str(path);
        }
        if !self.message.is_empty() {
            out.push('\n');
            out.push_str(&self.message);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_line_does_not_echo_a_secret() {
        let d = Decision::deny(
            RuleClass::Rule,
            Some("LEASH-SEC-001"),
            Some("src/config.ts"),
            "secret-shaped assignment in added lines; rotate if this was real",
        );
        let line = d.one_line();
        assert!(line.starts_with("deny RULE LEASH-SEC-001 path=src/config.ts"));
        assert!(!line.contains("sk-"));
        assert!(!line.contains("AKIA"));
    }
}
