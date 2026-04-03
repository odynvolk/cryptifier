//! Tests for ColinTalksCrypto Bitcoin Bull/Bear Index (CBBI) API integration

use crate::helpers;
use cryptifier::sources::cbbi::{calculate_average, parse_response};

#[tokio::test]
async fn test_parse_response_with_valid_data() {
    let data = helpers::load_fixture("cbbi");

    let result = parse_response(data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.index_value, 56);
}

#[tokio::test]
async fn test_parse_response_with_empty_data() {
    let empty_data: serde_json::Value = serde_json::json!({});
    let result = parse_response(empty_data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.index_value, 0);
}

#[tokio::test]
async fn test_parse_response_with_partial_data() {
    let partial_data: serde_json::Value = serde_json::json!({
        "PiCycle": {
            "1637971200": 0.5317,
            "1638057600": 0.531
        }
    });
    let result = parse_response(partial_data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    // Only one metric, so average should be around 53
    assert!(parsed.index_value > 50 && parsed.index_value < 60);
}

#[test]
fn test_calculate_average_with_full_data() {
    let data = helpers::load_fixture("cbbi");
    let result = calculate_average(&data);
    assert_eq!(result, 56);
}

#[test]
fn test_calculate_average_with_empty_data() {
    let data: serde_json::Value = serde_json::json!({});
    let result = calculate_average(&data);
    assert_eq!(result, 0);
}

#[test]
fn test_calculate_average_with_single_metric() {
    let data: serde_json::Value = serde_json::json!({
        "PiCycle": {
            "1637971200": 0.5317,
            "1638057600": 0.531
        }
    });
    let result = calculate_average(&data);
    // 0.53135 * 100 = 53.135, rounded to 53
    assert_eq!(result, 53);
}

#[test]
fn test_calculate_average_with_no_values() {
    let data: serde_json::Value = serde_json::json!({
        "PiCycle": {},
        "RUPL": {}
    });
    let result = calculate_average(&data);
    assert_eq!(result, 0);
}
