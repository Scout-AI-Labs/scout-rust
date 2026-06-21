use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::client::{Client, Json};
use crate::error::Error;

/// Multi-URL structured extraction.
pub struct Extract<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Extract::create`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExtractParams {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub find_via_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chars_total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

impl<'a> Extract<'a> {
    /// Extract structured data from one or more URLs.
    pub async fn create(&self, params: ExtractParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/extract", Some(&params), &[])
            .await
    }
}
