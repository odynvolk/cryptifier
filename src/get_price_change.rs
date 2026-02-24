use crate::common::PriceChange;
use crate::config::{get_currencies};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref LAST_FLOOR_PRICES: std::sync::Mutex<HashMap<String, f64>> = {
        let mut prices = HashMap::new();
        for currency in get_currencies() {
            prices.insert(currency.ticker.clone(), 0.0);
        }
        std::sync::Mutex::new(prices)
    };
}

fn parse_floor_price(price: f64) -> f64 {
    (price / 1000.0) * 1000.0
}

pub fn get_price_change(ticker: &str, price: f64, increment: i64) -> PriceChange {
    let current_floor_price = parse_floor_price(price);

    let mut last_floor_prices = LAST_FLOOR_PRICES.lock().unwrap();
    last_floor_prices.insert(ticker.to_string(), current_floor_price);

    if !last_floor_prices.contains_key(ticker) {
        return PriceChange::NoChange;
    }

    let last_price = last_floor_prices.get(ticker).unwrap();

    if last_floor_price_diff(*last_price, current_floor_price) > increment as f64 {
        if current_floor_price < *last_price {
            return PriceChange::Down;
        } else if current_floor_price > *last_price {
            return PriceChange::Up;
        }
    }

    PriceChange::NoChange
}

fn last_floor_price_diff(last: f64, current: f64) -> f64 {
    if last > current {
        last - current
    } else {
        current - last
    }
}
