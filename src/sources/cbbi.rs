use crate::cache::LONG_CACHE;
use crate::logger;

const METRICS: &[&str] = &[
    "PiCycle",
    "RUPL",
    "RHODL",
    "Puell",
    "2YMA",
    "Trolololo",
    "MVRV",
    "ReserveRisk",
    "Woobull",
    "Confidence",
];

fn calculate_average(data: &serde_json::Value) -> i64 {
    let mut result: i64 = 0;
    for metric in METRICS.iter() {
        if let Some(serde_json::Value::Array(metrics_array)) = data.get(metric) {
            if let Some(last) = metrics_array.last() {
                if let Some(val) = last.as_i64() {
                    result += val;
                }
            }
        }
    }
    (result / METRICS.len() as i64) * 100
}

pub async fn get_cbbi() -> i64 {
    let value = LONG_CACHE.get("cbbi");
    if let Some(value) = value {
        return value.parse::<i64>().unwrap_or(-1);
    }

    let client = reqwest::Client::new();

    match client
        .get("https://colintalkscrypto.com/cbbi/data/latest.json")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let result = calculate_average(&data);
                    LONG_CACHE.set("cbbi", result.to_string());
                    logger::debug(format!("Got CBBI from colintalkscrypto.com {}%", result).as_str());
                    result
                }
                Err(e) => {
                    logger::error(format!("Failed to parse CBBI response: {}", e).as_str());
                    -1
                }
            }
        }
        Err(e) => {
            logger::error(format!("Failed to get CBBI from colintalkscrypto.com: {}", e).as_str());
            -1
        }
    }
}
