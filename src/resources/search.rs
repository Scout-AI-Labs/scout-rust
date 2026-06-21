use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::client::{Client, Json};
use crate::error::Error;

/// Web search, agentic AI queries, and search-run history.
pub struct Search<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Search::create`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchParams {
    pub queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

/// Inputs to [`Search::ai_query`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct AiQueryParams {
    pub url: String,
    pub question: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,
}

impl<'a> Search<'a> {
    /// Run a web search.
    pub async fn create(&self, params: SearchParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/search", Some(&params), &[])
            .await
    }

    /// Answer a natural-language question by reading a page (and its links).
    pub async fn ai_query(&self, params: AiQueryParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/ai-query", Some(&params), &[])
            .await
    }

    /// List prior search runs.
    pub async fn list(&self, limit: u32, offset: u32) -> Result<Json, Error> {
        let query = [("limit", limit.to_string()), ("offset", offset.to_string())];
        self.client
            .request(Method::GET, "/v1/searches", None::<&Value>, &query)
            .await
    }

    /// Collect every search run across all pages.
    pub async fn list_all(&self) -> Result<Vec<Value>, Error> {
        super::paginate(self.client, "/v1/searches").await
    }

    /// Fetch a single search run by id.
    pub async fn get(&self, search_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/searches/{}", urlencode(search_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Cancel an in-flight search run.
    pub async fn cancel(&self, search_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/searches/{}/cancel", urlencode(search_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Fetch the event stream (as JSON) for a search run.
    pub async fn events(&self, search_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/searches/{}/events", urlencode(search_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }
}

/// Minimal percent-encoding for path segments (alnum and -._~ pass through).
pub(crate) fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
