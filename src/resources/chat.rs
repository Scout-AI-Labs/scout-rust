use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::client::{Client, Json};
use crate::error::Error;
use crate::stream::Stream;

/// OpenAI-compatible chat completions, optionally grounded with web search.
pub struct Chat<'a> {
    pub(crate) client: &'a Client,
}

/// A single message in a chat completion request.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Inputs to [`ChatCompletions::create`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatParams {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl<'a> Chat<'a> {
    /// Chat completions.
    pub fn completions(&self) -> ChatCompletions<'a> {
        ChatCompletions {
            client: self.client,
        }
    }
}

/// Creates chat completions.
pub struct ChatCompletions<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> ChatCompletions<'a> {
    /// Create a chat completion. Set `web_search` to ground in live results.
    pub async fn create(&self, params: ChatParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/chat/completions", Some(&params), &[])
            .await
    }

    /// Create a chat completion from a raw JSON body (advanced).
    pub async fn create_raw(&self, body: Value) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/chat/completions", Some(&body), &[])
            .await
    }

    /// Stream a chat completion as OpenAI-style chunks. Read token text from
    /// each event's `choices[0].delta.content`.
    pub async fn create_stream(&self, params: ChatParams) -> Result<Stream, Error> {
        let mut p = params;
        p.stream = Some(true);
        self.client
            .open_stream(Method::POST, "/v1/chat/completions", Some(&p))
            .await
    }
}
