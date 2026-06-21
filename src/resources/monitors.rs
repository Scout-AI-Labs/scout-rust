use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use super::search::urlencode;
use crate::client::{Client, Json};
use crate::error::Error;
use crate::stream::Stream;

/// Scheduled searches ("monitors"): run a query on a cadence, deliver via webhook.
pub struct Monitors<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Monitors::create`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct MonitorCreateParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Inputs to [`Monitors::update`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct MonitorUpdateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_prompt: Option<String>,
}

impl<'a> Monitors<'a> {
    /// Create a monitor with a query and a cadence or cron schedule.
    pub async fn create(&self, params: MonitorCreateParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/monitors", Some(&params), &[])
            .await
    }

    /// List monitors.
    pub async fn list(&self, limit: u32, offset: u32) -> Result<Json, Error> {
        let query = [("limit", limit.to_string()), ("offset", offset.to_string())];
        self.client
            .request(Method::GET, "/v1/monitors", None::<&Value>, &query)
            .await
    }

    /// Collect every monitor across all pages.
    pub async fn list_all(&self) -> Result<Vec<Value>, Error> {
        super::paginate(self.client, "/v1/monitors").await
    }

    /// Fetch a monitor by id.
    pub async fn get(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}", urlencode(monitor_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Update a monitor's query, schedule, or webhook.
    pub async fn update(
        &self,
        monitor_id: &str,
        params: MonitorUpdateParams,
    ) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}", urlencode(monitor_id));
        self.client
            .request(Method::PATCH, &path, Some(&params), &[])
            .await
    }

    /// Delete a monitor.
    pub async fn delete(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}", urlencode(monitor_id));
        self.client
            .request(Method::DELETE, &path, None::<&Value>, &[])
            .await
    }

    /// Pause a monitor.
    pub async fn pause(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}/pause", urlencode(monitor_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Resume a paused monitor.
    pub async fn resume(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}/resume", urlencode(monitor_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Trigger a monitor run immediately.
    pub async fn run(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}/run", urlencode(monitor_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Fetch a monitor's events.
    pub async fn events(&self, monitor_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/monitors/{}/events", urlencode(monitor_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Stream a monitor's events live (SSE).
    pub async fn stream_events(&self, monitor_id: &str) -> Result<Stream, Error> {
        let path = format!("/v1/monitors/{}/events", urlencode(monitor_id));
        self.client
            .open_stream(Method::GET, &path, None::<&Value>)
            .await
    }
}
