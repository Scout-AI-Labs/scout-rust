use reqwest::Method;
use serde::Serialize;

use crate::client::{Client, Json};
use crate::error::Error;

/// Company enrichment: profiles, logos, fonts, industry codes, styleguide.
pub struct Company<'a> {
    pub(crate) client: &'a Client,
}

/// Inputs to the domain-based company endpoints.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DomainParams {
    pub domain: String,
}

/// Inputs to [`Company::by_email`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ByEmailParams {
    pub email: String,
}

/// Inputs to [`Company::by_name`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ByNameParams {
    pub name: String,
}

/// Inputs to [`Company::by_ticker`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct ByTickerParams {
    pub ticker: String,
}

/// Inputs to [`Company::logo`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct LogoParams {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

impl<'a> Company<'a> {
    /// Full company profile from a domain.
    pub async fn enrich(&self, params: DomainParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company", Some(&params), &[])
            .await
    }

    /// Resolve a company from a work email address.
    pub async fn by_email(&self, params: ByEmailParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/by-email", Some(&params), &[])
            .await
    }

    /// Resolve a company from its name.
    pub async fn by_name(&self, params: ByNameParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/by-name", Some(&params), &[])
            .await
    }

    /// Resolve a company from a stock ticker.
    pub async fn by_ticker(&self, params: ByTickerParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/by-ticker", Some(&params), &[])
            .await
    }

    /// A condensed company profile (faster, fewer fields).
    pub async fn simple(&self, params: DomainParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/simple", Some(&params), &[])
            .await
    }

    /// Brand fonts detected on the company's site.
    pub async fn fonts(&self, params: DomainParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/fonts", Some(&params), &[])
            .await
    }

    /// Brand styleguide (colors, typography, logos) for a company.
    pub async fn styleguide(&self, params: DomainParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/styleguide", Some(&params), &[])
            .await
    }

    /// Company logo metadata.
    pub async fn logo(&self, params: LogoParams) -> Result<Json, Error> {
        self.client
            .request(Method::POST, "/v1/company/logo", Some(&params), &[])
            .await
    }
}
