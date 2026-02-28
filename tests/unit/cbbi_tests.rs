use crate::helpers;

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

fn calculate_average_from_fixture(data: &serde_json::Value) -> i64 {
    let mut result: f64 = 0.0;
    for metric in METRICS.iter() {
        if let Some(serde_json::Value::Object(metric_obj)) = data.get(metric) {
            // Get the last timestamp value
            if let Some(last_value) = metric_obj.values().last() {
                if let Some(val) = last_value.as_f64() {
                    result += val;
                }
            }
        }
    }
    ((result / METRICS.len() as f64) * 100.0) as i64
}

pub struct MockClient;

impl MockClient {
    pub async fn get_cbbi_fixture() -> i64 {
        let data = helpers::get_cbbi_fixture();
        calculate_average_from_fixture(&data)
    }
}

#[tokio::test]
async fn test_get_cbbi_from_fixture() {
    let result = MockClient::get_cbbi_fixture().await;
    assert_eq!(result, 56);
}

#[tokio::test]
async fn test_get_cbbi_fixture_structure() {
    let data = helpers::get_cbbi_fixture();

    for metric in METRICS.iter() {
        assert!(data.get(metric).is_some(), "Missing metric: {}", metric);
    }

    for metric in METRICS.iter() {
        if let Some(serde_json::Value::Object(metric_obj)) = data.get(metric) {
            let keys: Vec<_> = metric_obj.keys().collect();
            assert!(keys.iter().any(|k| *k == "1637971200"), "Missing timestamp in {}", metric);
            assert!(keys.iter().any(|k| *k == "1638057600"), "Missing timestamp in {}", metric);
        }
    }
}
