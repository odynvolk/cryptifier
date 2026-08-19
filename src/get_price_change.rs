//! Price change detection logic.
use crate::common::PriceChange;
use crate::config::get_currencies;
use crate::logger;
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
///
/// Returns `(PriceChange, f64)` where the `f64` is the **actual** measured
/// percentage change (signed), so callers can report the real move instead of
/// the configured threshold.
pub fn get_price_change(ticker: &str, price: f64, percentage_threshold: f64) -> (PriceChange, f64) {
    let mut last_prices = LAST_FLOOR_PRICES.lock().unwrap();

    if !last_prices.contains_key(ticker) {
        last_prices.insert(ticker.to_string(), price);
        return (PriceChange::NoChange, 0.0);
    }

    let last_price = *last_prices.get(ticker).unwrap();
    logger::debug(&format!("The last price for {} is {}", ticker, last_price));
    // Calculate percentage change
    let percent_change = if last_price > 0.0 {
        ((price - last_price) / last_price) * 100.0
    } else {
        last_prices.insert(ticker.to_string(), price);
        logger::debug(&format!("The last price for {} is updated to {}", ticker, price));

        0.0
    };

    logger::debug(&format!(
        "The price percentage change is {} and the threshold is {}",
        percent_change.abs(),
        percentage_threshold
    ));

    // Check if percentage change exceeds threshold (absolute value)
    if percent_change.abs() > percentage_threshold {
        last_prices.insert(ticker.to_string(), price);
        logger::debug(&format!("The last price for {} is updated to {}", ticker, price));

        if percent_change < 0.0 {
            return (PriceChange::Down, percent_change);
        } else if percent_change > 0.0 {
            return (PriceChange::Up, percent_change);
        }
    }

    (PriceChange::NoChange, percent_change)
}
