use reqwest::Method;
use serde::Serialize;

use crate::client::{Client, Json};
use crate::error::Error;

/// Whole-site operations: crawl and sitemap discovery.
pub struct Site<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Site::crawl`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct SiteCrawlParams {
    pub start_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_host_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_patterns: Option<Vec<String>>,
    #[serde(rename = "followSubdomains", skip_serializing_if = "Option::is_none")]
    pub follow_subdomains: Option<bool>,
}

/// Inputs to [`Site::map`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct SiteMapParams {
    pub start_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,
}

impl<'a> Site<'a> {
    /// Crawl a site from `start_url`.
    pub async fn crawl(&self, params: SiteCrawlParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/site/crawl", Some(&params), &[])
            .await
    }

    /// Discover a site's URLs (sitemap) from `start_url`.
    pub async fn map(&self, params: SiteMapParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/site/map", Some(&params), &[])
            .await
    }
}
