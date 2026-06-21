//! End-to-end tests against a wiremock server. Run with `cargo test`.

use scout_sdk::{ChatMessage, ChatParams, Client, DomainParams, SearchParams, SiteCrawlParams};
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

#[tokio::test]
async fn chat_stream_yields_deltas() {
    let server = MockServer::start().await;
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n\
                data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
                data: [DONE]\n\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let c = client(server.uri());
    let mut stream = c
        .chat()
        .completions()
        .create_stream(ChatParams {
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "hi".into(),
            }],
            ..Default::default()
        })
        .await
        .unwrap();

    let mut content = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        content.push_str(chunk["choices"][0]["delta"]["content"].as_str().unwrap());
    }
    assert_eq!(content, "Hello");
}

#[tokio::test]
async fn stream_events_yields_events() {
    let server = MockServer::start().await;
    let body = ": keepalive\n\n\
                event: run.progress\ndata: {\"type\":\"run.progress\"}\n\n\
                event: run.completed\ndata: {\"type\":\"run.completed\"}\n\n";
    Mock::given(method("GET"))
        .and(path("/v1/searches/abc/events"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let c = client(server.uri());
    let mut stream = c.search().stream_events("abc").await.unwrap();
    let mut types: Vec<String> = Vec::new();
    while let Some(evt) = stream.next().await {
        let evt = evt.unwrap();
        types.push(evt["type"].as_str().unwrap().to_string());
    }
    assert_eq!(types, vec!["run.progress", "run.completed"]);
}
