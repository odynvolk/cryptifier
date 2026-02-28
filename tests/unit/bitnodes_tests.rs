//! Tests for Bitnodes.io API integration

use crate::helpers;

/// Mock client for testing
pub struct MockClient;

impl MockClient {
    pub async fn get_bitnodes_fixture() -> String {
        let data = helpers::get_bitnodes_fixture();

        if let Some(serde_json::Value::Array(results_array)) = data.get("results") {
            if let Some(first) = results_array.first() {
                if let Some(total_nodes) = first.get("total_nodes") {
                    if let Some(nodes) = total_nodes.as_i64() {
                        return nodes.to_string();
                    }
                }
            }
        }
        "N/A".to_string()
    }
}

#[tokio::test]
async fn test_get_bitnodes_from_fixture() {
    let result = MockClient::get_bitnodes_fixture().await;
    assert_eq!(result, "16301");
}

#[tokio::test]
async fn test_get_bitnodes_fixture_structure() {
    let data = helpers::get_bitnodes_fixture();

    // Verify fixture structure matches expected API response
    assert!(data.get("count").is_some());
    assert!(data.get("results").is_some());

    let results = data.get("results").unwrap().as_array().unwrap();
    assert!(!results.is_empty());

    let first = &results[0];
    assert_eq!(first.get("total_nodes").unwrap().as_i64(), Some(16301));
    assert_eq!(first.get("latest_height").unwrap().as_i64(), Some(739023));
}
