use reqwest::Method;
use serde::Serialize;

use crate::client::{Client, Json};
use crate::error::Error;

/// Single-page operations: markdown, html, screenshot, images, extract.
pub struct Page<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Page::markdown`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct PageMarkdownParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<u32>,
}

/// Inputs to [`Page::html`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct PageHtmlParams {
    pub url: String,
}

/// Inputs to [`Page::screenshot`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct PageScreenshotParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_page: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismiss_overlays: Option<bool>,
}

/// Inputs to [`Page::images`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct PageImagesParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_images: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_data_uris: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Inputs to [`Page::extract`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct PageExtractParams {
    pub url: String,
}

impl<'a> Page<'a> {
    /// Fetch a page rendered to clean Markdown.
    pub async fn markdown(&self, params: PageMarkdownParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/page/markdown", Some(&params), &[])
            .await
    }

    /// Fetch a page's HTML.
    pub async fn html(&self, params: PageHtmlParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/page/html", Some(&params), &[])
            .await
    }

    /// Capture a screenshot of a page.
    pub async fn screenshot(&self, params: PageScreenshotParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/page/screenshot", Some(&params), &[])
            .await
    }

    /// Extract the images on a page.
    pub async fn images(&self, params: PageImagesParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/page/images", Some(&params), &[])
            .await
    }

    /// Structured extraction scoped to a single page.
    pub async fn extract(&self, params: PageExtractParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/page/extract", Some(&params), &[])
            .await
    }
}
