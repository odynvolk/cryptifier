//! Bitdis.org API integration for Bitcoin network statistics.
use crate::cache::LONG_CACHE;
use crate::cached_api::get_or_fetch_cached;

#[derive(Debug, Clone)]
pub struct BitdisData {
    pub total_nodes: i64,
}

pub async fn get_bitdis() -> String {
    get_or_fetch_cached(
        &LONG_CACHE,
        "bitdis",
        fetch_bitdis,
        parse_response,
        format_result,
        "Found {} Bitcoin nodes from bitdis.org",
        "Failed to parse bitdis response",
        "Failed to get nodes from bitdis.org: {}",
    )
    .await
}

pub async fn fetch_bitdis() -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://bitdis.org/api/live-data")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    resp.json().await
}

pub fn parse_response(data: serde_json::Value) -> Option<BitdisData> {
    let total_nodes = data.get("stats")?.get("total")?.as_i64()?;

    Some(BitdisData { total_nodes })
}

pub fn format_result(data: BitdisData) -> String {
    data.total_nodes.to_string()
}
