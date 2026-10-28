//! Embedded TypeScript sanitizer.
//!
//! Runs the compiled `rules/dist/sanitize.js` inside QuickJS (via `rquickjs`). QuickJS is
//! single-threaded and its `Context`/`Runtime` are `!Send`, so the engine lives on a dedicated
//! worker thread. Requests are handed to it over an mpsc channel and results come back over a
//! per-job reply channel. This keeps one warm engine (fast, low-memory) instead of rebuilding a
//! runtime per request.
//!
//! If the engine cannot initialize, or the bundle is missing, the factory in [`crate::sanitize`]
//! falls back to [`crate::sanitize::NativeSanitizer`] (fail-closed).

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

use serde_json::Value;

use crate::config::CustomRule;
use crate::sanitize::{NativeSanitizer, Sanitized, Sanitizer};

struct Job {
    input_json: String,
    reply: Sender<Result<String, String>>,
}

pub struct JsSanitizer {
    tx: Mutex<Sender<Job>>,
    opts: Value,
    fallback: NativeSanitizer,
}

/// Wrap the JSON input into a JS expression that calls the loaded `globalThis.sanitize`.
/// The JSON string is embedded as a JS string literal (JSON escaping is valid JS escaping).
fn build_wrapper(input_json: &str) -> String {
    let literal = serde_json::to_string(input_json).unwrap_or_else(|_| "\"{}\"".to_string());
    format!(
        "(function(){{var s={literal};var o=JSON.parse(s);\
         var r=globalThis.sanitize(o.messages||[],o.opts||{{}});\
         return JSON.stringify(r);}})()"
    )
}

fn parse_stripped(result: &Value) -> std::collections::BTreeMap<String, usize> {
    let mut out = std::collections::BTreeMap::new();
    if let Some(items) = result.get("stripped").and_then(|v| v.as_array()) {
        for item in items {
            let cat = item.get("category").and_then(|v| v.as_str());
            let count = item.get("count").and_then(|v| v.as_u64());
            if let (Some(cat), Some(count)) = (cat, count) {
                *out.entry(cat.to_string()).or_insert(0) += count as usize;
            }
        }
    }
    out
}

impl JsSanitizer {
    /// Spawn the engine thread, load the rules, and return `None` if initialization fails.
    pub fn new(source: String, strict: bool, custom: &[CustomRule]) -> Option<Self> {
        let opts = serde_json::json!({
            "strict": strict,
            "rules": custom
                .iter()
                .map(|c| serde_json::json!({
                    "pattern": c.pattern,
                    "replace": c.replace,
                    "flags": c.flags,
                }))
                .collect::<Vec<_>>(),
        });

        let (tx, rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        std::thread::Builder::new()
            .name("zrai-js".to_string())
            .spawn(move || run_engine(source, rx, ready_tx))
            .ok()?;

        match ready_rx.recv() {
            Ok(Ok(())) => Some(Self {
                tx: Mutex::new(tx),
                opts,
                fallback: NativeSanitizer::new(strict, custom),
            }),
            _ => None,
        }
    }
}

/// Worker-thread body. `Runtime` and `Context` are locals so the context can borrow the runtime.
fn run_engine(source: String, rx: Receiver<Job>, ready_tx: Sender<Result<(), String>>) {
    let rt = match rquickjs::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("runtime: {e}")));
            return;
        }
    };
    let ctx = match rquickjs::Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("context: {e}")));
            return;
        }
    };

    // Load the compiled rule bundle once; it assigns `globalThis.sanitize`.
    let loaded = ctx.with(|ctx| ctx.eval::<(), _>(source.as_str()).map_err(|e| e.to_string()));
    if let Err(e) = loaded {
        let _ = ready_tx.send(Err(format!("eval rules: {e}")));
        return;
    }
    let _ = ready_tx.send(Ok(()));

    while let Ok(job) = rx.recv() {
        let script = build_wrapper(&job.input_json);
        let result = ctx.with(|ctx| {
            ctx.eval::<String, _>(script.as_str())
                .map_err(|e| e.to_string())
        });
        let _ = job.reply.send(result);
    }
}

impl Sanitizer for JsSanitizer {
    fn sanitize(&self, body: &Value) -> Sanitized {
        let messages = match body.get("messages") {
            Some(m) => m.clone(),
            // No chat messages to scrub — leave the payload untouched.
            None => {
                return Sanitized {
                    value: body.clone(),
                    stripped: std::collections::BTreeMap::new(),
                }
            }
        };

        let input = serde_json::json!({ "messages": messages, "opts": self.opts.clone() });
        let input_json = match serde_json::to_string(&input) {
            Ok(s) => s,
            Err(_) => return self.fallback.sanitize(body),
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        let sent = match self.tx.lock() {
            Ok(tx) => tx.send(Job { input_json, reply: reply_tx }).is_ok(),
            Err(_) => false,
        };
        if !sent {
            return self.fallback.sanitize(body);
        }

        match reply_rx.recv() {
            Ok(Ok(json_str)) => match serde_json::from_str::<Value>(&json_str) {
                Ok(result) => {
                    let mut new_body = body.clone();
                    if let Some(m) = result.get("messages") {
                        new_body["messages"] = m.clone();
                    }
                    Sanitized {
                        value: new_body,
                        stripped: parse_stripped(&result),
                    }
                }
                Err(_) => self.fallback.sanitize(body),
            },
            _ => self.fallback.sanitize(body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rules_path() -> String {
        format!("{}/../../rules/dist/sanitize.js", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn embedded_rules_sanitize_paths_and_emails() {
        let path = rules_path();
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                // Precondition: build the bundle first (`cd rules && bun run build`).
                eprintln!("skipping embedded-rules test: {path} not built");
                return;
            }
        };
        let js = JsSanitizer::new(source, false, &[]).expect("js engine should initialize");
        let body = json!({
            "model": "gpt",
            "messages": [ { "role": "user", "content": "see /home/a/x.py and a@b.com" } ]
        });
        let out = js.sanitize(&body);
        assert_eq!(out.value["messages"][0]["content"], json!("see <path> and <email>"));
        assert_eq!(out.value["model"], json!("gpt"));
        assert_eq!(out.stripped.get("path"), Some(&1));
        assert_eq!(out.stripped.get("email"), Some(&1));
    }
}
