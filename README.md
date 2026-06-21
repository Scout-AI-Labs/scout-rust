# Scout Rust SDK

Official Rust SDK for the [Scout](https://usescout.sh) web-intelligence API — search, scrape, screenshot, extract, crawl, and company enrichment.

- **Minimal, audited dependencies.** `reqwest` + `serde` + `tokio` + `thiserror` — the idiomatic async core, nothing more.
- **Async + typed.** Builder-configured client, typed params, a `thiserror` error enum.
- **Resilient.** Automatic retries with backoff + jitter, configurable timeouts, idempotency keys.

## Requirements

- Rust 1.75+ and a Tokio runtime.

## Installation

```sh
cargo add scout-sdk
```

## Authentication

Create an API key in the [Scout dashboard](https://usescout.sh). The client reads `SCOUT_API_KEY` from the environment:

```rust
use scout_sdk::Client;

let client = Client::from_env()?;          // reads SCOUT_API_KEY
let client = Client::new("sk_...");        // or pass it explicitly
```

## Quickstart

```rust
use scout_sdk::{Client, SearchParams};

#[tokio::main]
async fn main() -> Result<(), scout_sdk::Error> {
    let client = Client::from_env()?;

    let res = client
        .search()
        .create(SearchParams {
            queries: vec!["best climate tech startups 2026".into()],
            depth: Some("standard".into()),
            country: Some("us".into()),
            ..Default::default()
        })
        .await?;

    println!("{res}");
    Ok(())
}
```

## Examples

```rust
use scout_sdk::{PageMarkdownParams, ExtractParams, DomainParams, SiteCrawlParams};

// Scrape a page to Markdown
let page = client.page().markdown(PageMarkdownParams { url: "https://example.com".into(), ..Default::default() }).await?;

// Structured extraction
let data = client.extract().create(ExtractParams {
    urls: vec!["https://example.com/pricing".into()],
    output_schema: Some(serde_json::json!({"type": "object"})),
    ..Default::default()
}).await?;

// Company enrichment
let company = client.company().enrich(DomainParams { domain: "stripe.com".into() }).await?;

// Crawl a site
let crawl = client.site().crawl(SiteCrawlParams { start_url: "https://example.com".into(), max_pages: Some(50), ..Default::default() }).await?;
```

## Error handling

Failures are a `scout_sdk::Error`. API errors carry `status`, `request_id`, `code`, and the parsed body:

```rust
match client.search().create(params).await {
    Ok(res) => { /* ... */ }
    Err(e) if e.is_rate_limited() => eprintln!("slow down"),
    Err(e) if e.is_authentication() => eprintln!("check your API key"),
    Err(e) => eprintln!("HTTP {:?} (req {:?}): {e}", e.status(), e.request_id()),
}
```

Helpers: `is_authentication` (401), `is_insufficient_credits` (402), `is_not_found` (404), `is_rate_limited` (429), `is_server_error` (5xx), plus `status()` and `request_id()`.

## Retries & timeouts

Transient failures (connection errors, timeouts, 408/409/429/5xx) are retried automatically — **2 times by default**, with exponential backoff + jitter, honoring `Retry-After`. Write methods send an auto-generated `Idempotency-Key`.

```rust
use std::time::Duration;

let client = Client::builder()
    .api_key("sk_...")
    .timeout(Duration::from_secs(30))
    .max_retries(4)
    .build()?;
```

## Pagination

```rust
let all_runs = client.search().list_all().await?; // walks every page
```

## Versioning

This SDK follows [SemVer](https://semver.org/) and sends the targeted Scout API version on every request; see [`CHANGELOG.md`](./CHANGELOG.md). API reference renders on [docs.rs](https://docs.rs/scout-sdk).

## Contributing

Issues and pull requests are welcome at [Scout-AI-Labs/scout-rust](https://github.com/Scout-AI-Labs/scout-rust).

## License

[MIT](./LICENSE)
