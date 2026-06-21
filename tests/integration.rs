//! End-to-end tests against a wiremock server. Run with `cargo test`.

use scout_sdk::{Client, DomainParams, SearchParams, SiteCrawlParams};
use serde_json::json;
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(base: String) -> Client {
    Client::builder()
        .api_key("sk_live_xyz")
        .base_url(base)
        .max_retries(3)
        .build()
        .unwrap()
}

#[tokio::test]
async fn post_round_trip_sends_auth_and_idempotency() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/search"))
        .and(header("authorization", "Bearer sk_live_xyz"))
        .and(header_exists("idempotency-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-request-id", "req_abc123")
                .set_body_json(json!({"ok": true})),
        )
        .mount(&server)
        .await;

    let res = client(server.uri())
        .search()
        .create(SearchParams {
            queries: vec!["hello world".into()],
            depth: Some("standard".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(res["ok"], true);
}

#[tokio::test]
async fn retries_on_500_then_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/site/crawl"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({"detail": "transient"})))
        .up_to_n_times(2)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/site/crawl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .with_priority(2)
        .mount(&server)
        .await;

    let res = client(server.uri())
        .site()
        .crawl(SiteCrawlParams {
            start_url: "https://example.com".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(res["ok"], true);
}

#[tokio::test]
async fn maps_401_to_typed_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/company"))
        .respond_with(
            ResponseTemplate::new(401)
                .insert_header("x-request-id", "req_abc123")
                .set_body_json(json!({"detail": "invalid api key"})),
        )
        .mount(&server)
        .await;

    let err = client(server.uri())
        .company()
        .enrich(DomainParams {
            domain: "x.com".into(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.status(), Some(401));
    assert_eq!(err.request_id(), Some("req_abc123"));
    assert!(err.is_authentication());
}

#[tokio::test]
async fn list_all_collects_a_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/searches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [{"id": 1}]})))
        .mount(&server)
        .await;

    let items = client(server.uri()).search().list_all().await.unwrap();
    assert_eq!(items.len(), 1);
}
