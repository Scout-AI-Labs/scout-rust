use reqwest::Method;
use serde::Serialize;

use crate::client::{Client, Json};
use crate::error::Error;

/// Product extraction from storefronts.
pub struct Products<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to [`Products::extract`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProductsParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(rename = "followSubdomains", skip_serializing_if = "Option::is_none")]
    pub follow_subdomains: Option<bool>,
}

/// Inputs to [`Products::one`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProductOneParams {
    pub url: String,
}

impl<'a> Products<'a> {
    /// Crawl a store and extract its products.
    pub async fn extract(&self, params: ProductsParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/products", Some(&params), &[])
            .await
    }

    /// Extract a single product from one product-detail URL.
    pub async fn one(&self, params: ProductOneParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/products/one", Some(&params), &[])
            .await
    }
}
