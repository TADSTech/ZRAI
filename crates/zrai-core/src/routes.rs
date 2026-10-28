//! Route mapping for the two supported API shapes.
//!
//! ZRAI must be a drop-in for both OpenAI-style and Anthropic-style clients, so it accepts
//! the two canonical paths and forwards them to the matching upstream route.

pub struct Route {
    /// Upstream path (identical to the client path for both supported shapes).
    pub path: &'static str,
    /// Default upstream base used when the config does not override it.
    pub default_base: &'static str,
}

/// Map a client path to an upstream route, or `None` if unsupported.
pub fn route(client_path: &str) -> Option<Route> {
    match client_path {
        "/v1/chat/completions" => Some(Route {
            path: "/v1/chat/completions",
            default_base: "https://api.openai.com",
        }),
        "/v1/messages" => Some(Route {
            path: "/v1/messages",
            default_base: "https://api.anthropic.com",
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_openai_chat_completions() {
        let r = route("/v1/chat/completions").expect("route");
        assert_eq!(r.path, "/v1/chat/completions");
        assert_eq!(r.default_base, "https://api.openai.com");
    }

    #[test]
    fn maps_anthropic_messages() {
        let r = route("/v1/messages").expect("route");
        assert_eq!(r.path, "/v1/messages");
        assert_eq!(r.default_base, "https://api.anthropic.com");
    }

    #[test]
    fn rejects_unknown_routes() {
        assert!(route("/v1/embeddings").is_none());
        assert!(route("/health").is_none());
    }
}
