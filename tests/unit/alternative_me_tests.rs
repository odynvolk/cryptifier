//! Tests for Alternative.me Fear & Greed Index API integration

use crate::helpers;
use cryptifier::sources::alternative_me::{format_result, parse_response, FearGreedData};

#[tokio::test]
async fn test_parse_response_with_valid_data() {
    let data = helpers::load_fixture("alternative-me");

    let result = parse_response(data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.value_classification, "Extreme Fear");
    assert_eq!(parsed.value_num, 23);
}

#[tokio::test]
async fn test_parse_response_with_empty_data() {
    let empty_data: serde_json::Value = serde_json::json!({ "data": [] });
    let result = parse_response(empty_data);
    assert!(result.is_none());
}

#[test]
fn test_format_result() {
    let data = FearGreedData {
        value_classification: "Extreme Fear".to_string(),
        value_num: 23,
    };
    let result = format_result(data);
    assert_eq!(result, "\"Extreme Fear\" | 23");
}
