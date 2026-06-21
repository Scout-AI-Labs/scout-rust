use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use super::search::urlencode;
use crate::client::{Client, Json};
use crate::error::Error;

/// Async tasks ("jobs"): submit a task, then poll or stream events.
pub struct Jobs<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Jobs::create`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct JobCreateParams {
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl<'a> Jobs<'a> {
    /// Submit a job. The result includes a task id to poll with `get`.
    pub async fn create(&self, params: JobCreateParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/jobs", Some(&params), &[])
            .await
    }

    /// List jobs.
    pub async fn list(&self, limit: u32, offset: u32) -> Result<Json, Error> {
        let query = [("limit", limit.to_string()), ("offset", offset.to_string())];
        self.client
            .request(Method::GET, "/v1/jobs", None::<&Value>, &query)
            .await
    }

    /// Collect every job across all pages.
    pub async fn list_all(&self) -> Result<Vec<Value>, Error> {
        super::paginate(self.client, "/v1/jobs").await
    }

    /// Fetch a job by task id.
    pub async fn get(&self, task_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/jobs/{}", urlencode(task_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Cancel a running job.
    pub async fn cancel(&self, task_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/jobs/{}/cancel", urlencode(task_id));
        self.client
            .request(Method::POST, &path, None::<&Value>, &[])
            .await
    }

    /// Fetch a job's events.
    pub async fn events(&self, task_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/jobs/{}/events", urlencode(task_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Start a run for a job.
    pub async fn start_run(&self, body: Value) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/jobs/runs", Some(&body), &[])
            .await
    }

    /// Fetch the result of a completed run.
    pub async fn run_result(&self, run_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/jobs/runs/{}", urlencode(run_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }

    /// Fetch a run's events.
    pub async fn run_events(&self, run_id: &str) -> Result<Json, Error> {
        let path = format!("/v1/jobs/runs/{}/events", urlencode(run_id));
        self.client
            .request(Method::GET, &path, None::<&Value>, &[])
            .await
    }
}
