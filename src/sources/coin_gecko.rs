//! CoinGecko API integration for cryptocurrency prices.
use crate::cache::SHORT_CACHE;
use crate::logger;

/// Price data for a cryptocurrency from CoinGecko.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CoinPrice {
    pub usd: Option<f64>,
    pub usd_24h_vol: Option<f64>,
}

/// Fetches the current price of a cryptocurrency from CoinGecko.
pub async fn get_ticker(id: &str) -> Option<std::collections::HashMap<String, CoinPrice>> {
    let cache_key = format!("coin_gecko_{}", id);
    let cached = SHORT_CACHE.get(&cache_key);
    if let Some(cached) = cached {
        if let Ok(data) = serde_json::from_str::<std::collections::HashMap<String, CoinPrice>>(&cached) {
            logger::debug(
                format!(
                    "Got ticker {} from CoinGecko (cached) ${}",
                    id,
                    data.get(id).and_then(|p| p.usd).unwrap_or(0.0)
                )
                .as_str(),
            );
            return Some(data);
        }
    }

    match fetch_ticker(id).await {
        Ok(data) => {
            if let Some(cached_value) = serde_json::to_string(&data).ok() {
                SHORT_CACHE.set(&cache_key, cached_value);
            }
            logger::debug(
                format!(
                    "Got ticker {} from CoinGecko ${}",
                    id,
                    data.get(id).and_then(|p| p.usd).unwrap_or(0.0)
                )
                .as_str(),
            );
            Some(data)
        }
        Err(e) => {
            logger::error(format!("Failed to get ticker {} from CoinGecko: {}", id, e).as_str());
            None
        }
    }
}

/// Fetches raw JSON response from the CoinGecko API.
pub async fn fetch_ticker(id: &str) -> Result<std::collections::HashMap<String, CoinPrice>, reqwest::Error> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_vol=true",
            id
        ))
        .header("content-type", "application/json")
        .header("User-Agent", "Cryptifier/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    resp.json().await
}
