//! ColinTalksCrypto Bitcoin Bull/Bear Index (CBBI) API integration.
use crate::cache::LONG_CACHE;
use crate::cached_api::get_or_fetch_cached;

/// Available metrics used to calculate the CBBI.
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

/// Struct to hold parsed CBBI data
#[derive(Debug, Clone)]
pub struct CbbiData {
    pub index_value: i64,
}

/// Calculates the average of all available metrics from the CBBI data.
pub fn calculate_average(data: &serde_json::Value) -> i64 {
    let mut result: f64 = 0.0;
    let mut count = 0;
    for metric in METRICS.iter() {
        if let Some(serde_json::Value::Object(metric_obj)) = data.get(metric) {
            if let Some(last_value) = metric_obj.values().next_back() {
                if let Some(val) = last_value.as_f64() {
                    result += val;
                    count += 1;
                }
            }
        }
    }

    let average = if count > 0 {
        result / count as f64
    } else {
        0.0
    };
    (average * 100.0) as i64
}

/// Fetches the current Bitcoin Bull/Bear Index (CBBI) from ColinTalksCrypto.
pub async fn get_cbbi() -> i64 {
    get_or_fetch_cached(
        &LONG_CACHE,
        "cbbi",
        fetch_cbbi,
        parse_response,
        |data: CbbiData| data.index_value.to_string(),
        "Got CBBI from colintalkscrypto.com {}%",
        "Failed to parse CBBI response",
        "Failed to get CBBI from colintalkscrypto.com: {}",
    )
    .await
    .parse::<i64>()
    .unwrap_or(-1)
}

/// Fetches raw JSON response from the CBBI API.
pub async fn fetch_cbbi() -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://colintalkscrypto.com/cbbi/data/latest.json")
        .header("User-Agent", "Cryptifier/1.0")
        .header("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    resp.json().await
}

/// Parses the raw JSON response into structured CBBI data.
pub fn parse_response(data: serde_json::Value) -> Option<CbbiData> {
    let index_value = calculate_average(&data);
    Some(CbbiData { index_value })
}
