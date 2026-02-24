use crate::cache::LONG_CACHE;
use crate::logger;

pub async fn get_bitnodes() -> String {
    let value = LONG_CACHE.get("bitnodes");
    if let Some(value) = value {
        return value;
    }

    let client = reqwest::Client::new();

    match client
        .get("https://bitnodes.io/api/v1/snapshots/?limit=1")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    if let Some(serde_json::Value::Array(results_array)) = data.get("results") {
                        if let Some(first) = results_array.first() {
                            if let Some(total_nodes) = first.get("total_nodes") {
                                if let Some(nodes) = total_nodes.as_i64() {
                                    let nodes_str = nodes.to_string();
                                    LONG_CACHE.set("bitnodes", nodes_str.clone());
                                    logger::debug(format!("Got reachable nodes from bitnodes.io {}", nodes).as_str());
                                    return nodes_str;
                                }
                            }
                        }
                    }
                    "N/A".to_string()
                }
                Err(e) => {
                    logger::error(format!("Failed to parse bitnodes response: {}", e).as_str());
                    "N/A".to_string()
                }
            }
        }
        Err(e) => {
            logger::error(format!("Failed to get reachable nodes from bitnodes.io: {}", e).as_str());
            "N/A".to_string()
        }
    }
}
