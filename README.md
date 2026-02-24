# Cryptifier

Fetches...

1. Price of crypto currencies from [https://www.coingecko.com/](https://colintalkscrypto.com/)
2. Number of reachable Bitcoin nodes from [https://bitnodes.io/](https://bitnodes.io/)
3. CBBI from [https://colintalkscrypto.com/](https://colintalkscrypto.com/)
3. Fear and Greed index from [https://alternative.me/](https://alternative.me/)

Notifies users of price going up or down in increments via...

1. Telegram via a TelegramBot

## Structure

```
src/tests/
├── mod.rs              # Test module exports
├── README.md           # This file
├── mock_data.rs        # Mock data structures for all APIs
├── coin_gecko_tests.rs # Tests for CoinGecko API
├── alternative_me_tests.rs # Tests for Alternative.me API
├── bitnodes_tests.rs   # Tests for Bitnodes.io API
└── cbbi_tests.rs       # Tests for ColinTalksCrypto CBBI API
```

## Configuration

Create a .env file with values needed in your setup.

```
APP__LOG_LEVEL=debug
APP__NOTIFIER_SLEEP=300
APP__CURRENCIES=[{"ticker": "bitcoin","increment": 3000}]
APP__TELEGRAM_API_KEY=<key>
APP__TELEGRAM_CHAT_IDS=<id>
APP__TELEGRAM_GET_UPDATES=false
```

## Running Tests

To run all tests:

```bash
cargo test
```

To run tests for a specific module:

```bash
cargo test tests::coin_gecko
```

## Test Structure

Each test file contains:

1. **Mock data structures** - Helper structs that generate mock API responses (defined in `mock_data.rs`)
2. **Mock clients** - Helper structs that process mock data without making network calls
3. **Test functions** - Individual test cases that verify the parsing logic

## Mock Data Providers

- **CoinGeckoMock** - Mocks CoinGecko API responses for cryptocurrency prices
- **AlternativeMeMock** - Mocks Alternative.me Fear & Greed Index responses
- **BitnodesMock** - Mocks Bitnodes.io node count responses
- **CbbiMock** - Mocks ColinTalksCrypto CBBI responses

## Shared Helpers

- `../helpers.rs` - Test fixture loader that reads JSON files from `tests/data/`

## Example Test

```rust
#[tokio::test]
async fn test_get_ticker_with_mock() {
    let mut mock = CoinGeckoMock::new();
    mock.with_price("bitcoin", 65000);

    let result = MockClient::get_ticker_mock("bitcoin", &mock).await;
    assert!(result.is_some());
    let data = result.unwrap();
    let bitcoin_price = data.get("bitcoin").unwrap();
    assert_eq!(bitcoin_price.usd, Some(65000));
}
```

## Adding New Tests

1. Add mock data to `mock_data.rs` if needed
2. Create a new test file or add to existing test files
3. Use `#[cfg(test)]` to ensure tests only compile in test mode
4. Follow the pattern: mock data → mock client → test functions
