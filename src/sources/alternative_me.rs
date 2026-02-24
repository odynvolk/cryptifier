use crate::cache::LONG_CACHE;
use crate::logger;

pub async fn get_fear_greed_index() -> String {
    let value = LONG_CACHE.get("f&gi");
    if let Some(value) = value {
        return value;
    }

    let client = reqwest::Client::new();

    match client
        .get("https://api.alternative.me/fng/?limit=1&format=json")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let result = if let Some(serde_json::Value::Array(data_array)) = data.get("data") {
                        if let Some(first) = data_array.first() {
                            let value_class = first.get("value_classification")
                                .and_then(|v| v.as_str())
                                .unwrap_or("N/A");
                            let value_num = first.get("value")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<i64>().ok())
                                .unwrap_or(0);
                            format!("\"{}\" | {}", value_class, value_num)
                        } else {
                            "N/A".to_string()
                        }
                    } else {
                        "N/A".to_string()
                    };

                    LONG_CACHE.set("f&gi", result.clone());
                    logger::debug(format!("Got F&GI from api.alternative.me {}", result).as_str());
                    result
                }
                Err(e) => {
                    logger::error(format!("Failed to parse fear & greed response: {}", e).as_str());
                    "N/A".to_string()
                }
            }
        }
        Err(e) => {
            logger::error(format!("Failed to get F&GI from api.alternative.me: {}", e).as_str());
            "N/A".to_string()
        }
    }
}
