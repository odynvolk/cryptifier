//! Bitnodes.io API integration for Bitcoin network statistics.
use crate::cache::LONG_CACHE;
use crate::logger;

/// Struct to hold parsed Bitnodes data
#[derive(Debug, Clone)]
pub struct BitnodesData {
    pub total_nodes: i64,
}

/// Fetches the current number of reachable Bitcoin nodes from Bitnodes.io.
pub async fn get_bitnodes() -> String {
    let value = LONG_CACHE.get("bitnodes");
    if let Some(value) = value {
        return value.clone();
    }

    match fetch_bitnodes().await {
        Ok(data) => match parse_response(data) {
            Some(parsed) => {
                let result = format_result(parsed);
                LONG_CACHE.set("bitnodes", result.clone());
                logger::debug(format!("Got nodes from bitnodes.io {}", result).as_str());
                result
            }
            None => {
                logger::error("Failed to parse bitnodes response");
                "N/A".to_string()
            }
        },
        Err(e) => {
            logger::error(format!("Failed to get nodes from bitnodes.io: {}", e).as_str());
            "N/A".to_string()
        }
    }
}

/// Fetches raw JSON response from the Bitnodes.io API.
pub async fn fetch_bitnodes() -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://bitnodes.io/api/v1/snapshots/?limit=1")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    resp.json().await
}

/// Parses the raw JSON response into structured Bitnodes data.
pub fn parse_response(data: serde_json::Value) -> Option<BitnodesData> {
    let results_array = data.get("results")?.as_array()?;
    let first = results_array.first()?;

    let total_nodes = first.get("total_nodes").and_then(|v| v.as_i64()).unwrap_or(0);

    Some(BitnodesData { total_nodes })
}

/// Formats the parsed Bitnodes data into a user-friendly string.
pub fn format_result(data: BitnodesData) -> String {
    data.total_nodes.to_string()
}
