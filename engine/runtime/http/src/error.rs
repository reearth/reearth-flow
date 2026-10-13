use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum EgressError {
    #[error("Invalid URL '{url}': {message}")]
    InvalidUrl { url: String, message: String },

    #[error("URL scheme '{0}' is not allowed; only http and https are supported")]
    SchemeNotAllowed(String),

    #[error("{0}")]
    Blocked(String),

    #[error("Failed to create HTTP client: {0}")]
    Build(#[source] reqwest::Error),

    #[error("HTTP request failed: {}", with_sources(.0))]
    Request(#[source] reqwest::Error),
}

/// `reqwest::Error` only displays its outermost layer; the reason a request
/// was blocked by the resolver or the redirect policy lives in its sources.
pub(crate) fn with_sources(e: &reqwest::Error) -> String {
    let mut message = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        message.push_str(": ");
        message.push_str(&s.to_string());
        source = s.source();
    }
    message
}
