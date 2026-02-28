use cryptifier::sources::coin_gecko::CoinPrice;
use crate::helpers;

pub struct MockClient;

impl MockClient {
    pub async fn get_ticker_fixture(id: &str) -> Option<std::collections::HashMap<String, CoinPrice>> {
        helpers::get_coin_gecko_fixture(id)
    }
}

#[tokio::test]
async fn test_get_ticker_fixture_bitcoin() {
    let result = MockClient::get_ticker_fixture("bitcoin").await;
    assert!(result.is_some());
    let data = result.unwrap();
    let bitcoin_price = data.get("bitcoin").unwrap();
    assert_eq!(bitcoin_price.usd, Some(66674.0));
}

#[tokio::test]
async fn test_get_ticker_fixture_not_available() {
    let result = MockClient::get_ticker_fixture("nonexistent").await;
    assert!(result.is_none());
}
