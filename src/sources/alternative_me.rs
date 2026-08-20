//! Alternative.me Fear & Greed Index API integration.
use crate::cache::LONG_CACHE;
use crate::cached_api::get_or_fetch_cached;

/// Struct to hold parsed Fear & Greed Index data
use crate::http::shared_client;
#[derive(Debug, Clone)]
pub struct FearGreedData {
    pub value_classification: String,
    pub value_num: i64,
}

/// Fetches the current Fear & Greed Index from Alternative.me.
pub async fn get_fear_greed_index() -> String {
    get_or_fetch_cached(
        &LONG_CACHE,
        "f&gi",
        fetch_fear_greed,
        parse_response,
        format_result,
        "Got F&GI from api.alternative.me {}",
        "Failed to parse fear & greed response",
        "Failed to get F&GI from api.alternative.me: {}",
    )
    .await
}

/// Fetches raw JSON response from the Alternative.me API.
pub async fn fetch_fear_greed() -> Result<serde_json::Value, reqwest::Error> {
    let client = shared_client();

    let resp = client
        .get("https://api.alternative.me/fng/?limit=1&format=json")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    resp.json().await
}

/// Parses the raw JSON response into structured Fear & Greed data.
pub fn parse_response(data: serde_json::Value) -> Option<FearGreedData> {
    let data_array = data.get("data")?.as_array()?;
    let first = data_array.first()?;

    let value_class = first
        .get("value_classification")
        .and_then(|v| v.as_str())
        .unwrap_or("N/A")
        .to_string();

    let value_num = first
        .get("value")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    Some(FearGreedData {
        value_classification: value_class,
        value_num,
    })
}

/// Formats the parsed Fear & Greed data into a user-friendly string.
pub fn format_result(data: FearGreedData) -> String {
    format!("\"{}\" | {}", data.value_classification, data.value_num)
}
