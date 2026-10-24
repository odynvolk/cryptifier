//! Tests for the shared cached-API fetch/parse/format pipeline.

use cryptifier::cache::Cache;
use cryptifier::cached_api::{get_or_fetch_cached, LogMessages};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The log-message bundle passed to every `get_or_fetch_cached` call.
fn test_messages() -> LogMessages {
    LogMessages {
        success_log_msg: "fetched {} in tests",
        parse_error_msg: "test parse failed",
        fetch_error_msg: "test fetch failed: {}",
    }
}

#[derive(Debug)]
struct Stats {
    total: i64,
}

fn parse_stats(data: serde_json::Value) -> Option<Stats> {
    Some(Stats {
        total: data.get("stats")?.get("total")?.as_i64()?,
    })
}

fn format_stats(stats: Stats) -> String {
    stats.total.to_string()
}

#[tokio::test]
async fn test_cache_miss_fetches_parses_and_formats() {
    static CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(600));

    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/live-data")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stats":{"total":1234}}"#)
        .create_async()
        .await;

    let url = format!("{}/live-data", server.url());
    let fetch = move || {
        let client = reqwest::Client::new();
        let url = url.clone();
        async move { client.get(&url).send().await?.json::<serde_json::Value>().await }
    };

    let result = get_or_fetch_cached(
        &CACHE,
        "cached_api_miss",
        fetch,
        parse_stats,
        format_stats,
        test_messages(),
    )
    .await;

    assert_eq!(result, "1234");
    mock.assert_async().await;
    // The miss must also populate the cache for the next caller.
    assert_eq!(CACHE.get("cached_api_miss").as_deref(), Some("1234"));
}

#[tokio::test]
async fn test_cache_hit_skips_fetch_entirely() {
    static CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(600));

    CACHE.set("cached_api_hit", "cached-value".to_string());

    let fetch_calls = AtomicUsize::new(0);
    let fetch = || {
        fetch_calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(serde_json::json!({"stats": {"total": 9999}})) }
    };

    let result = get_or_fetch_cached(
        &CACHE,
        "cached_api_hit",
        fetch,
        parse_stats,
        format_stats,
        test_messages(),
    )
    .await;

    assert_eq!(result, "cached-value");
    assert_eq!(fetch_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_parse_failure_returns_na_and_does_not_cache() {
    static CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(600));

    let fetch = || async { Ok(serde_json::json!({"unexpected": "shape"})) };

    let result = get_or_fetch_cached(
        &CACHE,
        "cached_api_parse_fail",
        fetch,
        parse_stats,
        format_stats,
        test_messages(),
    )
    .await;

    assert_eq!(result, "N/A");
    assert!(CACHE.get("cached_api_parse_fail").is_none());
}

#[tokio::test]
async fn test_fetch_failure_returns_na() {
    static CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(600));

    let mut server = mockito::Server::new_async().await;
    // A 500 with a non-JSON body makes `send()` succeed but `json()` fail.
    let mock = server
        .mock("GET", "/broken")
        .with_status(500)
        .with_body("not json")
        .create_async()
        .await;

    let url = format!("{}/broken", server.url());
    let fetch = move || {
        let client = reqwest::Client::new();
        let url = url.clone();
        async move { client.get(&url).send().await?.json::<serde_json::Value>().await }
    };

    let result = get_or_fetch_cached(
        &CACHE,
        "cached_api_fetch_fail",
        fetch,
        parse_stats,
        format_stats,
        test_messages(),
    )
    .await;

    assert_eq!(result, "N/A");
    mock.assert_async().await;
    assert!(CACHE.get("cached_api_fetch_fail").is_none());
}
