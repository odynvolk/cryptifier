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

/// Rounds a price down to the nearest thousand.
pub fn parse_floor_price(price: f64) -> f64 {
    (price / 1000.0).floor() * 1000.0
}

/// Detects if a cryptocurrency price has changed by a significant amount.
pub fn get_price_change(ticker: &str, price: f64, increment: i64) -> PriceChange {
    let current_floor_price = parse_floor_price(price);

    let mut last_floor_prices = LAST_FLOOR_PRICES.lock().unwrap();
    if !last_floor_prices.contains_key(ticker) {
        last_floor_prices.insert(ticker.to_string(), current_floor_price);
        return PriceChange::NoChange;
    }

    let last_price = *last_floor_prices.get(ticker).unwrap();
    last_floor_prices.insert(ticker.to_string(), current_floor_price);

    if last_floor_price_diff(last_price, current_floor_price) > increment as f64 {
        if current_floor_price < last_price {
            return PriceChange::Down;
        } else if current_floor_price > last_price {
            return PriceChange::Up;
        }
    }

    PriceChange::NoChange
}

/// Calculates the absolute difference between two floor prices.
fn last_floor_price_diff(last: f64, current: f64) -> f64 {
    if last > current {
        last - current
    } else {
        current - last
    }
}
