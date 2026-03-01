use crate::cache::SHORT_CACHE;
use crate::logger;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CoinPrice {
    pub usd: Option<f64>,
}

pub async fn get_ticker(id: &str) -> Option<std::collections::HashMap<String, CoinPrice>> {
    let cache_key = format!("coin_gecko_{}", id);
    let cached = SHORT_CACHE.get(&cache_key);
    if let Some(cached) = cached {
        if let Ok(data) = serde_json::from_str::<std::collections::HashMap<String, CoinPrice>>(&cached) {
            logger::debug(format!("Got ticker {} from CoinGecko (cached) ${}", id, data.get(id).and_then(|p| p.usd).unwrap_or(0.0)).as_str());
            return Some(data);
        }
    }

    let client = reqwest::Client::new();

    match client
        .get(format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            id
        ))
        .header("content-type", "application/json")
        .header("User-Agent", "Cryptifier/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<std::collections::HashMap<String, CoinPrice>>().await {
                Ok(data) => {
                    let cached_value = serde_json::to_string(&data).ok()?;
                    SHORT_CACHE.set(&cache_key, cached_value);
                    logger::debug(format!("Got ticker {} from CoinGecko ${}", id, data.get(id).and_then(|p| p.usd).unwrap_or(0.0)).as_str());
                    Some(data)
                }
                Err(e) => {
                    logger::error(format!("Failed to get ticker {} due to CoinGecko response: {}", id, e).as_str());
                    None
                }
            }
        }
        Err(e) => {
            logger::error(format!("Failed to get ticker {} from CoinGecko: {}", id, e).as_str());
            None
        }
    }
}
