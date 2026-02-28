//! Tests for Alternative.me Fear & Greed Index API integration

use crate::helpers;

/// Mock client for testing
pub struct MockClient;

impl MockClient {
    pub async fn get_fng_fixture() -> String {
        let data = helpers::get_fng_fixture();
        serde_json::to_string(&data).unwrap()
    }
}

#[tokio::test]
async fn test_get_fng_from_fixture() {
    let json_response = MockClient::get_fng_fixture().await;
    let data: serde_json::Value = serde_json::from_str(&json_response).unwrap();

    assert!(data.get("data").is_some());
    let data_array = data.get("data").unwrap().as_array().unwrap();
    assert!(!data_array.is_empty());
    let first = &data_array[0];
    assert_eq!(first.get("value_classification").unwrap().as_str(), Some("Extreme Fear"));
}

#[tokio::test]
async fn test_get_fng_fixture_values() {
    let json_response = MockClient::get_fng_fixture().await;
    let data: serde_json::Value = serde_json::from_str(&json_response).unwrap();
    let data_array = data.get("data").unwrap().as_array().unwrap();
    let first = &data_array[0];

    assert_eq!(first.get("value").unwrap().as_str(), Some("23"));
    assert_eq!(first.get("value_classification").unwrap().as_str(), Some("Extreme Fear"));
    assert_eq!(first.get("timestamp").unwrap().as_str(), Some("1641772800"));
}
