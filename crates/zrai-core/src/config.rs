//! Configuration.
//!
//! The config file is the **only** disk read in the whole system, and it happens exactly
//! once at startup — never per request. This preserves the "zero disk writes during a
//! session" invariant (see `docs/architecture.md`).

use serde::Deserialize;
use std::path::Path;

/// A user-supplied sanitization rule (mirrors the TypeScript `CustomRule`).
#[derive(Debug, Clone, Deserialize)]
pub struct CustomRule {
    pub pattern: String,
    pub replace: String,
    #[serde(default)]
    pub flags: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Upstream base URL (e.g. `https://openrouter.ai/api`). When unset, a per-route
    /// default is used (OpenAI for chat/completions, Anthropic for messages).
    pub upstream: Option<String>,
    /// Listen port (default 4040).
    pub port: Option<u16>,
    /// When true, also scrub lower-precision env-secret assignments.
    #[serde(default)]
    pub strict: bool,
    /// Provider ZDR parameters merged into the root payload (e.g.
    /// `{ provider = { zdr = true } }` for OpenRouter).
    pub zdr: Option<serde_json::Value>,
    /// Extra user rules applied after built-ins.
    #[serde(default)]
    pub rules: Vec<CustomRule>,
}

impl Config {
    /// Load from a TOML file if present, else return defaults.
    /// This is a startup-only disk read; it is never called per request.
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        let cfg = toml::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(cfg)
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(4040)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_missing() {
        let cfg = Config::load("does-not-exist.toml").unwrap();
        assert_eq!(cfg.port(), 4040);
        assert!(cfg.upstream.is_none());
        assert!(!cfg.strict);
    }

    #[test]
    fn parses_zdr_and_rules() {
        let toml = r#"
            upstream = "https://openrouter.ai/api"
            strict = true
            [zdr.provider]
            zdr = true
            [[rules]]
            pattern = "CODENAME=\\w+"
            replace = "CODENAME=<redacted>"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.port(), 4040);
        assert!(cfg.strict);
        assert_eq!(cfg.rules.len(), 1);
        assert_eq!(cfg.zdr.unwrap()["provider"]["zdr"], serde_json::json!(true));
    }
}
