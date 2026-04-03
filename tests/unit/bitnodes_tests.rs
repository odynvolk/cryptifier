//! Tests for Bitnodes.io API integration

use crate::helpers;
use cryptifier::sources::bitnodes::{format_result, parse_response, BitnodesData};

#[tokio::test]
async fn test_parse_response_with_valid_data() {
    let data = helpers::load_fixture("bitnodes");

    let result = parse_response(data);

    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.total_nodes, 16301);
}

#[tokio::test]
async fn test_parse_response_with_empty_data() {
    let empty_data: serde_json::Value = serde_json::json!({ "results": [] });
    let result = parse_response(empty_data);
    assert!(result.is_none());
}

#[test]
fn test_format_result() {
    let data = BitnodesData { total_nodes: 16301 };
    let result = format_result(data);
    assert_eq!(result, "16301");
}
