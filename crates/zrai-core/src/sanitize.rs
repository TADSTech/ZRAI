//! Sanitization abstraction and the native (Rust) fallback.
//!
//! The primary sanitizer is the TypeScript rule layer in `rules/`, loaded via an embedded JS
//! engine (added in a follow-up). This module defines the [`Sanitizer`] trait both backends
//! implement, plus [`NativeSanitizer`] — a compact, fail-closed fallback so the proxy never
//! forwards unsanitized context if the JS bundle is unavailable.
//!
//! The native rules intentionally mirror only the highest-signal categories (paths, emails,
//! IPv4, bearer tokens, API keys, PEM keys). The TypeScript layer is the full source of truth.

use crate::config::CustomRule;
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;

/// Result of sanitizing a request body: the rewritten payload plus per-category counts.
pub struct Sanitized {
    pub value: Value,
    pub stripped: BTreeMap<String, usize>,
}

/// Sanitizes an outbound request body before egress.
pub trait Sanitizer: Send + Sync {
    fn sanitize(&self, body: &Value) -> Sanitized;
}

struct Rule {
    category: &'static str,
    regex: Regex,
    marker: &'static str,
}

/// Compact Rust sanitizer used as a fail-closed fallback.
pub struct NativeSanitizer {
    rules: Vec<Rule>,
    custom: Vec<(Regex, String)>,
}

impl NativeSanitizer {
    pub fn new(strict: bool, custom: &[CustomRule]) -> Self {
        let mut rules = vec![
            Rule {
                category: "path",
                regex: Regex::new(r#"(?:/home/|/Users/|/var/|/root/|[A-Za-z]:\\(?:Users|home)\\)[^\s"'`]*"#)
                    .expect("valid path regex"),
                marker: "<path>",
            },
            Rule {
                category: "email",
                regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
                    .expect("valid email regex"),
                marker: "<email>",
            },
            Rule {
                category: "ipv4",
                regex: Regex::new(
                    r"\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b",
                )
                .expect("valid ipv4 regex"),
                marker: "<ipv4>",
            },
            Rule {
                category: "bearer_token",
                regex: Regex::new(r"\bBearer\s+[A-Za-z0-9\-._~+/]+=*")
                    .expect("valid bearer regex"),
                marker: "Bearer <token>",
            },
            Rule {
                category: "api_key",
                regex: Regex::new(r"\b(?:sk|pk|rk|org)-[A-Za-z0-9]{16,}")
                    .expect("valid api key regex"),
                marker: "<api-key>",
            },
            Rule {
                category: "ssh_key",
                regex: Regex::new(
                    r"-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----[\s\S]*?-----END (?:[A-Z ]+ )?PRIVATE KEY-----",
                )
                .expect("valid ssh key regex"),
                marker: "<private-key>",
            },
        ];

        if strict {
            rules.push(Rule {
                category: "env_secret",
                regex: Regex::new(
                    r"(?i)\b(?:API_?KEY|SECRET|TOKEN|PASSWORD|PASSWD|PRIVATE_?KEY|ACCESS_?KEY|CLIENT_?SECRET)[A-Z0-9_]*\s*=\s*\S+",
                )
                .expect("valid env secret regex"),
                marker: "<secret>",
            });
        }

        // Compile user rules, skipping any invalid pattern instead of crashing at startup.
        let custom = custom
            .iter()
            .filter_map(|c| {
                let mut flags = c.flags.clone().unwrap_or_default();
                // Rust regex has no 'g' flag; keep only supported inline flags.
                flags.retain(|ch| matches!(ch, 'i' | 'm' | 's' | 'u' | 'x'));
                let pattern = if flags.is_empty() {
                    c.pattern.clone()
                } else {
                    format!("(?{flags}){}", c.pattern)
                };
                Regex::new(&pattern).ok().map(|re| (re, c.replace.clone()))
            })
            .collect();

        Self { rules, custom }
    }

    fn redact(&self, input: &str) -> (String, BTreeMap<String, usize>) {
        let mut out = input.to_string();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();

        for rule in &self.rules {
            let n = rule.regex.find_iter(&out).count();
            if n > 0 {
                *counts.entry(rule.category.to_string()).or_insert(0) += n;
                out = rule.regex.replace_all(&out, rule.marker).into_owned();
            }
        }

        for (re, replacement) in &self.custom {
            let n = re.find_iter(&out).count();
            if n > 0 {
                *counts.entry("custom".to_string()).or_insert(0) += n;
                out = re.replace_all(&out, replacement.as_str()).into_owned();
            }
        }

        (out, counts)
    }
}

impl Sanitizer for NativeSanitizer {
    fn sanitize(&self, body: &Value) -> Sanitized {
        let mut value = body.clone();
        let mut stripped: BTreeMap<String, usize> = BTreeMap::new();

        if let Some(messages) = value.get_mut("messages").and_then(|m| m.as_array_mut()) {
            for msg in messages {
                match msg.get_mut("content") {
                    Some(Value::String(s)) => {
                        let (out, counts) = self.redact(s);
                        for (k, v) in counts {
                            *stripped.entry(k).or_insert(0) += v;
                        }
                        *s = out;
                    }
                    Some(Value::Array(blocks)) => {
                        for block in blocks {
                            if let Some(Value::String(t)) = block.get_mut("text") {
                                let (out, counts) = self.redact(t);
                                for (k, v) in counts {
                                    *stripped.entry(k).or_insert(0) += v;
                                }
                                *t = out;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Sanitized { value, stripped }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sanitizer() -> NativeSanitizer {
        NativeSanitizer::new(false, &[])
    }

    #[test]
    fn redacts_paths_and_emails() {
        let body = json!({ "messages": [ { "role": "user", "content": "see /home/a/x.py and a@b.com" } ] });
        let out = sanitizer().sanitize(&body);
        assert_eq!(out.value["messages"][0]["content"], json!("see <path> and <email>"));
        assert_eq!(out.stripped.get("path"), Some(&1));
        assert_eq!(out.stripped.get("email"), Some(&1));
    }

    #[test]
    fn handles_anthropic_text_blocks() {
        let body = json!({ "messages": [ { "role": "user", "content": [ { "type": "text", "text": "open C:\\Users\\b\\k.pem" } ] } ] });
        let out = sanitizer().sanitize(&body);
        assert_eq!(out.value["messages"][0]["content"][0]["text"], json!("open <path>"));
    }

    #[test]
    fn redacts_bearer_tokens() {
        let body = json!({ "messages": [ { "role": "user", "content": "Authorization: Bearer sk-abcdef1234567890" } ] });
        let out = sanitizer().sanitize(&body);
        assert_eq!(
            out.value["messages"][0]["content"],
            json!("Authorization: Bearer <token>")
        );
    }

    #[test]
    fn leaves_unrelated_fields_intact() {
        let body = json!({ "model": "gpt", "messages": [], "temperature": 0.2 });
        let out = sanitizer().sanitize(&body);
        assert_eq!(out.value["model"], json!("gpt"));
        assert_eq!(out.value["temperature"], json!(0.2));
    }

    #[test]
    fn no_messages_is_a_noop() {
        let body = json!({ "model": "gpt" });
        let out = sanitizer().sanitize(&body);
        assert_eq!(out.value, json!({ "model": "gpt" }));
        assert!(out.stripped.is_empty());
    }
}
