use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use super::search::urlencode;
use crate::client::{Client, Json};
use crate::error::Error;
use crate::stream::Stream;

/// Find-all ("lists"): build a list of entities matching a query, then enrich
/// or extend the run.
pub struct Lists<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Lists::create`] and [`Lists::run`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListsParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl<'a> Lists<'a> {
    /// Run a find-all synchronously.
    pub async fn create(&self, params: ListsParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/lists", Some(&params), &[])
            .await
    }

    /// Start an async find-all run; poll `runs().get(id)` for progress.
    pub async fn run(&self, params: ListsParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/lists/runs", Some(&params), &[])
            .await
    }

    /// Operations on async find-all runs.
    pub fn runs(&self) -> ListRuns<'a> {
        ListRuns {
            client: self.client,
        }
    }
}

/// Operations on async find-all runs.
pub struct ListRuns<'a> {
    pub(crate) client: &'a Client,
}

impl<'a> ListRuns<'a> {
    /// List find-all runs.
    pub async fn list(&self, limit: u32, offset: u32) -> Result<Json, Error> {
        let query = [("limit", limit.to_string()), ("offset", offset.to_string())];
        self.client
            .request(Method::GET, "/v1/lists/runs", None::<&Value>, &query)
            .await
    }

    /// Collect every find-all run across all pages.
    pub async fn list_all(&self) -> Result<Vec<Value>, Error> {
        super::paginate(self.client, "/v1/lists/runs").await
    }

    /// Fetch a find-all run by id.
    pub async fn get(&self, findall_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/lists/runs/{}", urlencode(findall_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Cancel a find-all run.
    pub async fn cancel(&self, findall_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/lists/runs/{}/cancel", urlencode(findall_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Enrich the run's entities with additional fields.
    pub async fn enrich(&self, findall_id: &str, body: Value) -> Result<Json, Error> {
        let path = format!("/v1/lists/runs/{}/enrich", urlencode(findall_id));
        self.client
            .request(Method::POST, &path, Some(&body), &[])
            .await
    }

    /// Extend the run with more matching entities.
    pub async fn extend(&self, findall_id: &str, body: Value) -> Result<Json, Error> {
        let path = format!("/v1/lists/runs/{}/extend", urlencode(findall_id));
        self.client
            .request(Method::POST, &path, Some(&body), &[])
            .await
    }

    /// Fetch a find-all run's events.
    pub async fn events(&self, findall_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/lists/runs/{}/events", urlencode(findall_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Stream a find-all run's progress events live (SSE).
    pub async fn stream_events(&self, findall_id: &str) -> Result<Stream, Error> {
        let path = format!("/v1/lists/runs/{}/events", urlencode(findall_id));
        self.client
            .open_stream(Method::GET, &path, None::<&Value>)
            .await
    }
}
