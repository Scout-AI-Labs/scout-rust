//! Error type for the Scout SDK.

use serde_json::Value;

/// The Scout REST API version this SDK targets.
pub const API_VERSION: &str = "2026-06-21";

/// Errors returned by the SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A non-2xx HTTP response. Inspect `status` or use the `is_*` helpers.
    #[error("scout api error: HTTP {status}: {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Human-readable message parsed from the response body.
        message: String,
        /// Machine-readable code from the body, if any.
        code: Option<String>,
        /// Server-assigned request id (from `x-request-id`), for support.
        request_id: Option<String>,
        /// Parsed JSON error body, if any.
        body: Option<Value>,
        /// Seconds from a `Retry-After` header, if the server sent one.
        retry_after: Option<f64>,
    },

    /// A transport-level failure (DNS, refused connection, timeout, TLS).
    #[error("scout connection error: {0}")]
    Connection(#[from] reqwest::Error),

    /// The response body could not be decoded as expected.
    #[error("scout decode error: {0}")]
    Decode(String),

    /// The client was misconfigured (e.g. a missing API key).
    #[error("scout config error: {0}")]
    Config(String),
}

impl Error {
    /// The HTTP status code, when this is an API error.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// The request id, when available.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Error::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }

    /// Whether the request should be retried.
    pub(crate) fn is_retriable(&self) -> bool {
        match self {
            Error::Connection(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            Error::Api { status, .. } => {
                matches!(status, 408 | 409 | 429 | 500 | 502 | 503 | 504)
            }
            _ => false,
        }
    }

    /// 401 - missing or invalid API key.
    pub fn is_authentication(&self) -> bool {
        self.status() == Some(401)
    }

    /// 402 - the team is out of credits.
    pub fn is_insufficient_credits(&self) -> bool {
        self.status() == Some(402)
    }

    /// 404 - the resource does not exist.
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }

    /// 429 - rate limit exceeded.
    pub fn is_rate_limited(&self) -> bool {
        self.status() == Some(429)
    }

    /// 5xx - the server failed to handle a valid request.
    pub fn is_server_error(&self) -> bool {
        matches!(self.status(), Some(s) if s >= 500)
    }
}

/// Build the most specific [`Error::Api`] from a status, body, and headers.
pub(crate) fn api_error(
    status: u16,
    request_id: Option<String>,
    retry_after: Option<String>,
    body: Option<Value>,
) -> Error {
    let message = error_message(body.as_ref(), status);
    let code = error_code(body.as_ref());
    let retry_after = retry_after.and_then(|s| s.trim().parse::<f64>().ok());
    Error::Api {
        status,
        message,
        code,
        request_id,
        body,
        retry_after,
    }
}

fn error_message(body: Option<&Value>, status: u16) -> String {
    if let Some(Value::Object(map)) = body {
        for key in ["detail", "error", "message"] {
            match map.get(key) {
                Some(Value::String(s)) => return s.clone(),
                Some(Value::Object(inner)) => {
                    if let Some(Value::String(s)) = inner.get("message") {
                        return s.clone();
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(Value::String(s)) = body {
        if !s.is_empty() {
            return s.clone();
        }
    }
    format!("Scout API returned HTTP {status}")
}

fn error_code(body: Option<&Value>) -> Option<String> {
    if let Some(Value::Object(map)) = body {
        if let Some(Value::String(s)) = map.get("code") {
            return Some(s.clone());
        }
        if let Some(Value::Object(err)) = map.get("error") {
            if let Some(Value::String(s)) = err.get("code") {
                return Some(s.clone());
            }
        }
    }
    None
}
