use crate::cache::Cache;
use crate::logger;
use reqwest::Error;
use serde_json::Value;
use std::future::Future;

/// The log messages emitted by `get_or_fetch_cached` across its three outcome paths.
#[derive(Debug, Clone, Copy)]
pub struct LogMessages {
    /// Logged on success, with the first `{}` replaced by the formatted result.
    pub success_log_msg: &'static str,
    /// Logged when the raw response cannot be parsed.
    pub parse_error_msg: &'static str,
    /// Logged when the fetch fails, with the first `{}` replaced by the error.
    pub fetch_error_msg: &'static str,
}

/// Generic function to fetch data from an API with caching, parsing, and formatting.
///
/// Returns the formatted value, or `"N/A"` when the fetch or the parse fails.
///
/// # Arguments
///
/// * `cache` - The cache to use (typically LONG_CACHE)
/// * `cache_key` - The key to store/retrieve data in the cache
/// * `fetch` - A function that returns a future resolving to the raw JSON response
/// * `parse` - A function that parses the JSON into a typed value
/// * `format` - A function that formats the typed value into a display string
/// * `messages` - Log messages for the success, parse-error and fetch-error paths
pub async fn get_or_fetch_cached<T, F, Fut, P, Fmt>(
    cache: &'static Cache<String>,
    cache_key: &'static str,
    fetch: F,
    parse: P,
    format: Fmt,
    messages: LogMessages,
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
                    // Replace the first occurrence of "{}" in the success message with the result
                    let success_msg = messages.success_log_msg.replacen("{}", &result, 1);
                    logger::debug(&success_msg);
                    result
                }
                None => {
                    logger::error(messages.parse_error_msg);
                    "N/A".to_string()
                }
            }
        }
        Err(e) => {
            // Replace the first occurrence of "{}" in the fetch error message with the error
            let error_msg = messages.fetch_error_msg.replacen("{}", &e.to_string(), 1);
            logger::error(&error_msg);
            "N/A".to_string()
        }
    }
}
