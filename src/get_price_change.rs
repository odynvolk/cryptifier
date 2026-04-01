//! Price change detection logic.
use crate::common::PriceChange;
use crate::config::get_currencies;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Thread-safe cache of last observed floor prices for each currency.
static LAST_FLOOR_PRICES: Lazy<Mutex<HashMap<String, f64>>> = Lazy::new(|| {
    let mut prices = HashMap::new();
    for currency in get_currencies() {
        prices.insert(currency.ticker.clone(), 0.0);
    }
    Mutex::new(prices)
});

/// Detects if a cryptocurrency price has changed by a significant percentage.
pub fn get_price_change(ticker: &str, price: f64, percentage_threshold: f64) -> PriceChange {
    let mut last_prices = LAST_FLOOR_PRICES.lock().unwrap();

    if !last_prices.contains_key(ticker) {
        last_prices.insert(ticker.to_string(), price);
        return PriceChange::NoChange;
    }

    let last_price = *last_prices.get(ticker).unwrap();
    last_prices.insert(ticker.to_string(), price);

    // Calculate percentage change
    let percent_change = if last_price > 0.0 {
        ((price - last_price) / last_price) * 100.0
    } else {
        0.0
    };

    // Check if percentage change exceeds threshold (absolute value)
    if percent_change.abs() > percentage_threshold {
        if percent_change < 0.0 {
            return PriceChange::Down;
        } else if percent_change > 0.0 {
            return PriceChange::Up;
        }
    }

    PriceChange::NoChange
}
