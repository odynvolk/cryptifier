//! Tests for Bitdis.org API integration
use cryptifier::sources::bitdis::{format_result, parse_response, BitdisData};

#[tokio::test]
async fn test_parse_response_with_valid_data() {
    let data = crate::helpers::load_fixture("bitdis");

    let result = parse_response(data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.total_nodes, 27022);
}

#[tokio::test]
async fn test_parse_response_with_missing_stats() {
    let empty_data: serde_json::Value = serde_json::json!({});
    let result = parse_response(empty_data);
    assert!(result.is_none());
}

#[tokio::test]
async fn test_parse_response_with_null_total() {
    let data: serde_json::Value = serde_json::json!({ "stats": { "total": null } });
    let result = parse_response(data);
    assert!(result.is_none());
}

#[test]
fn test_format_result() {
    let data = BitdisData { total_nodes: 27022 };
    let result = format_result(data);
    assert_eq!(result, "27022");
}
