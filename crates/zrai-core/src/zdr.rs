//! ZDR (Zero Data Retention) parameter injection.
//!
//! Different providers express "do not retain" differently. Rather than hard-coding per
//! provider, the config supplies a JSON fragment that is deep-merged into the request root
//! (e.g. `{ provider = { zdr = true } }` for OpenRouter).

use serde_json::Value;

/// Deep-merge `src` into `dst`. Objects merge recursively; scalars/arrays replace.
fn merge(dst: &mut Value, src: &Value) {
    match (dst, src) {
        (Value::Object(d), Value::Object(s)) => {
            for (k, v) in s {
                merge(d.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (d, s) => *d = s.clone(),
    }
}

/// Inject the configured ZDR fragment into the outbound payload, if any.
pub fn inject(body: &mut Value, zdr: Option<&Value>) {
    if let Some(extra) = zdr {
        merge(body, extra);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merges_nested_provider() {
        let mut body = json!({ "model": "x", "messages": [] });
        inject(&mut body, Some(&json!({ "provider": { "zdr": true } })));
        assert_eq!(body["provider"]["zdr"], json!(true));
        assert_eq!(body["model"], json!("x"));
    }

    #[test]
    fn preserves_existing_provider_fields() {
        let mut body = json!({ "provider": { "order": ["a"] } });
        inject(&mut body, Some(&json!({ "provider": { "zdr": true } })));
        assert_eq!(body["provider"]["order"], json!(["a"]));
        assert_eq!(body["provider"]["zdr"], json!(true));
    }

    #[test]
    fn noop_when_none() {
        let mut body = json!({ "a": 1 });
        inject(&mut body, None);
        assert_eq!(body, json!({ "a": 1 }));
    }
}
