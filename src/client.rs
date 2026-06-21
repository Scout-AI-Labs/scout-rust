//! Core HTTP client for the Scout API.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::error::{api_error, Error, API_VERSION};
use crate::resources::{
    Chat, Company, Extract, Jobs, Lists, Monitors, Page, Products, Search, Site,
};

/// The version of this SDK crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_BASE_URL: &str = "https://core.usescout.sh";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_MAX_RETRIES: u32 = 2;

/// A decoded JSON response. Fields vary by endpoint.
pub type Json = Value;

/// Client for the Scout web-intelligence API.
///
/// ```no_run
/// # async fn run() -> Result<(), scout_sdk::Error> {
/// let client = scout_sdk::Client::from_env()?;
/// let res = client.search().create(scout_sdk::SearchParams {
///     queries: vec!["climate tech startups".into()],
///     ..Default::default()
/// }).await?;
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    max_retries: u32,
}

impl Client {
    /// Build a client with an explicit API key and default configuration.
    pub fn new(api_key: impl Into<String>) -> Self {
        ClientBuilder::new()
            .api_key(api_key)
            .build()
            .expect("valid client")
    }

    /// Build a client reading the API key from `SCOUT_API_KEY`.
    pub fn from_env() -> Result<Self, Error> {
        let key = std::env::var("SCOUT_API_KEY")
            .map_err(|_| Error::Config("SCOUT_API_KEY is not set".into()))?;
        ClientBuilder::new().api_key(key).build()
    }

    /// Start building a customized client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Web search, agentic AI queries, and search-run history.
    pub fn search(&self) -> Search<'_> {
        Search { client: self }
    }
    /// Single-page operations: markdown, html, screenshot, images, extract.
    pub fn page(&self) -> Page<'_> {
        Page { client: self }
    }
    /// Multi-URL structured extraction.
    pub fn extract(&self) -> Extract<'_> {
        Extract { client: self }
    }
    /// Company enrichment: profiles, logos, fonts, industry codes, styleguide.
    pub fn company(&self) -> Company<'_> {
        Company { client: self }
    }
    /// Find-all ("lists").
    pub fn lists(&self) -> Lists<'_> {
        Lists { client: self }
    }
    /// Product extraction from storefronts.
    pub fn products(&self) -> Products<'_> {
        Products { client: self }
    }
    /// Whole-site operations: crawl and sitemap discovery.
    pub fn site(&self) -> Site<'_> {
        Site { client: self }
    }
    /// Async tasks ("jobs").
    pub fn jobs(&self) -> Jobs<'_> {
        Jobs { client: self }
    }
    /// Scheduled searches ("monitors").
    pub fn monitors(&self) -> Monitors<'_> {
        Monitors { client: self }
    }
    /// OpenAI-compatible chat completions.
    pub fn chat(&self) -> Chat<'_> {
        Chat { client: self }
    }

    /// Issue a request with retries and typed error mapping. Internal.
    pub(crate) async fn request<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        query: &[(&str, String)],
    ) -> Result<Json, Error> {
        let url = format!("{}{}", self.base_url, path);
        let body_bytes = match body {
            Some(b) => Some(serde_json::to_vec(b).map_err(|e| Error::Decode(e.to_string()))?),
            None => None,
        };
        let is_write = method != Method::GET;
        // One idempotency key, reused across retries so they stay safe.
        let idempotency_key = if is_write { Some(new_id()) } else { None };

        let mut attempt: u32 = 0;
        loop {
            let result = self
                .send_once(
                    &method,
                    &url,
                    body_bytes.as_deref(),
                    query,
                    idempotency_key.as_deref(),
                )
                .await;
            match result {
                Ok(value) => return Ok(value),
                Err(err) => {
                    if !err.is_retriable() || attempt >= self.max_retries {
                        return Err(err);
                    }
                    tokio::time::sleep(backoff(attempt, &err)).await;
                    attempt += 1;
                }
            }
        }
    }

    async fn send_once(
        &self,
        method: &Method,
        url: &str,
        body_bytes: Option<&[u8]>,
        query: &[(&str, String)],
        idempotency_key: Option<&str>,
    ) -> Result<Json, Error> {
        let mut req = self
            .http
            .request(method.clone(), url)
            .bearer_auth(&self.api_key)
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, format!("scout-rust/{VERSION}"))
            .header("Scout-Version", API_VERSION);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(bytes) = body_bytes {
            req = req
                .header(CONTENT_TYPE, "application/json")
                .body(bytes.to_vec());
        }
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }

        let resp = req.send().await?; // reqwest::Error -> Error::Connection
        let status = resp.status().as_u16();
        let request_id = header(&resp, "x-request-id");
        let retry_after = header(&resp, "retry-after");
        let bytes = resp.bytes().await?;

        let parsed: Option<Value> = if bytes.is_empty() {
            None
        } else {
            Some(serde_json::from_slice(&bytes).unwrap_or(Value::Null))
        };

        if (200..300).contains(&status) {
            return Ok(parsed.unwrap_or(Value::Null));
        }
        Err(api_error(status, request_id, retry_after, parsed))
    }
}

fn header(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

/// Builder for [`Client`].
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl ClientBuilder {
    fn new() -> Self {
        ClientBuilder {
            api_key: std::env::var("SCOUT_API_KEY").ok(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Override the API origin.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the per-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the number of automatic retries for transient failures.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Build the [`Client`].
    pub fn build(self) -> Result<Client, Error> {
        let api_key = self.api_key.filter(|k| !k.is_empty()).ok_or_else(|| {
            Error::Config("missing API key; set SCOUT_API_KEY or use .api_key()".into())
        })?;
        let http = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(Error::Connection)?;
        Ok(Client {
            http,
            base_url: self.base_url.trim_end_matches('/').to_string(),
            api_key,
            max_retries: self.max_retries,
        })
    }
}

fn backoff(attempt: u32, err: &Error) -> Duration {
    if let Error::Api {
        retry_after: Some(secs),
        ..
    } = err
    {
        return Duration::from_secs_f64(secs.min(60.0).max(0.0));
    }
    let base_ms = (500u64 * (1u64 << attempt)).min(8_000);
    Duration::from_millis((base_ms as f64 * jitter()) as u64)
}

/// Pseudo-random factor in [0.5, 1.0) derived from the clock, avoiding a
/// dependency on `rand` just for jitter.
fn jitter() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    0.5 + (f64::from(nanos % 1_000) / 1_000.0) * 0.5
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique idempotency key (clock nanos + a process-local counter).
fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("idmp-{nanos:x}-{n:x}")
}
