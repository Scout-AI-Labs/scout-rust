//! Resource groups - a faithful 1:1 mirror of the REST API tags.

use reqwest::Method;
use serde_json::Value;

use crate::client::Client;
use crate::error::Error;

mod chat;
mod company;
mod extract;
mod jobs;
mod lists;
mod monitors;
mod page;
mod products;
mod search;
mod site;

pub use chat::{Chat, ChatCompletions, ChatMessage, ChatParams};
pub use company::{ByEmailParams, ByNameParams, ByTickerParams, Company, DomainParams, LogoParams};
pub use extract::{Extract, ExtractParams};
pub use jobs::{JobCreateParams, Jobs};
pub use lists::{ListRuns, Lists, ListsParams};
pub use monitors::{MonitorCreateParams, MonitorUpdateParams, Monitors};
pub use page::{
    Page, PageExtractParams, PageHtmlParams, PageImagesParams, PageMarkdownParams,
    PageScreenshotParams,
};
pub use products::{ProductOneParams, Products, ProductsParams};
pub use search::{AiQueryParams, Search, SearchParams};
pub use site::{Site, SiteCrawlParams, SiteMapParams};

const COMMON_ITEM_KEYS: [&str; 7] = [
    "items", "data", "results", "searches", "runs", "jobs", "monitors",
];

/// Pull the array of records out of a list response of unknown shape.
pub(crate) fn extract_items(payload: &Value) -> Vec<Value> {
    if let Value::Array(arr) = payload {
        return arr.clone();
    }
    if let Value::Object(map) = payload {
        for key in COMMON_ITEM_KEYS {
            if let Some(Value::Array(arr)) = map.get(key) {
                return arr.clone();
            }
        }
        for value in map.values() {
            if let Value::Array(arr) = value {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

/// Walk every page of an offset-paginated endpoint and collect the items.
pub(crate) async fn paginate(client: &Client, path: &str) -> Result<Vec<Value>, Error> {
    let limit: usize = 50;
    let mut offset: usize = 0;
    let mut out: Vec<Value> = Vec::new();
    loop {
        let query = [("limit", limit.to_string()), ("offset", offset.to_string())];
        let page = client
            .request(Method::GET, path, None::<&Value>, &query)
            .await?;
        let items = extract_items(&page);
        let n = items.len();
        out.extend(items);
        if n < limit {
            break;
        }
        offset += n;
    }
    Ok(out)
}
