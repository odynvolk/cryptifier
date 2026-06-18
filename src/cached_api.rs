use crate::cache::{Cache, LONG_CACHE};
use crate::logger;
use futures::Future;
use once_cell::sync::Lazy;
use pin_project_lite::pin_project;
use reqwest::Error;
use serde_json::Value;
use std::pin::Pin;
use std::sync::LazyLock;

// We'll use a static reference to LONG_CACHE, but note that LONG_CACHE is already a Lazy<Cache<String>>
// We can use it directly.

/// Generic function to fetch data from an API with caching, parsing, and formatting.
///
/// # Arguments
///
/// * `cache` - The cache to use (typically LONG_CACHE)
// * `cache_key` - The key to store/retrieve data in the cache
// * `fetch` - A function that returns a future resolving to the raw JSON response
// * `parse` - A function that parses the JSON into a typed value
// * `format` - A function that formats the typed value into a display string
// * `success_log_msg` - Message to log on success (will be formatted with the result string)
// * `parse_error_msg` - Message to log if parsing fails
// * `fetch_error_msg` - Message to log if fetching fails (will be formatted with the error)
pub async fn get_or_fetch_cached<T, F, Fut, P, Fmt>(
    cache: &'static Cache<String>,
    cache_key: &'static str,
    fetch: F,
    parse: P,
    format: Fmt,
    success_log_msg: &'static str,
    parse_error_msg: &'static str,
    fetch_error_msg: &'static str,
) -> String
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Value, Error>> + Send,
    P: Fn(Value) -> Option<T>,
    Fmt: Fn(T) -> String,
{
    // Try to get from cache first
    if let Some(value) = cache.get(cache_key) {
        return value;
    }

    // Fetch from API
    let fetch_result = fetch().await;
    match fetch_result {
        Ok(data) => {
            // Parse the response
            match parse(data) {
                Some(parsed) => {
                    // Format and cache the result
                    let result = format(parsed);
                    cache.set(cache_key, result.clone());
                    logger::debug(&format!(success_log_msg, result));
                    result
                }
                None => {
                    logger::error(parse_error_msg);
                    "N/A".to_string()
                }
            }
        }
        Err(e) => {
            logger::error(&format!(fetch_error_msg, e));
            "N/A".to_string()
        }
    }
}