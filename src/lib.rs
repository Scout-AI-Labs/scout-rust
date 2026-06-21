//! Official Rust SDK for the [Scout](https://usescout.sh) web-intelligence API:
//! search, scrape, screenshot, extract, crawl, and company enrichment.
//!
//! ```no_run
//! # async fn run() -> Result<(), scout_sdk::Error> {
//! use scout_sdk::{Client, SearchParams};
//!
//! let client = Client::from_env()?; // reads SCOUT_API_KEY
//! let res = client
//!     .search()
//!     .create(SearchParams {
//!         queries: vec!["climate tech startups".into()],
//!         ..Default::default()
//!     })
//!     .await?;
//! println!("{res}");
//! # Ok(()) }
//! ```

#![forbid(unsafe_code)]

mod client;
mod error;
mod resources;
mod stream;

pub use client::{Client, ClientBuilder, Json, VERSION};
pub use error::{Error, API_VERSION};
pub use resources::*;
pub use stream::Stream;
