//! Main notification orchestration and price monitoring loop.
use crate::common::PriceChange;
use crate::config::{get_currencies, get_notifier_sleep, is_quiet_hours, is_quiet_mode_enabled};
use crate::get_price_change::get_price_change;
use crate::logger;
use crate::notifiers::telegram;
use crate::sources::alternative_me;
use crate::sources::bitdis;
use crate::sources::cbbi;
use crate::sources::coin_gecko;

pub fn to_upper_case(ticker: &str) -> String {
    let mut chars = ticker.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.collect::<String>(),
    }
}

pub fn price_change_as_text(change: &PriceChange) -> String {
    match change {
        PriceChange::Up => "🟢 <b>up</b>".to_string(),
        PriceChange::Down => "🔴 <b>down</b>".to_string(),
        PriceChange::NoChange => "🟡 no change".to_string(),
    }
}

/// Builds the price-change notification body (pure, no I/O).
pub fn format_price_notification(
    ticker: &str,
    change: &PriceChange,
    percent_change: f64,
    price: f64,
) -> String {
    format!(
        "💰 <b>{}</b> is {} {:.2}%! ${}",
        ticker,
        price_change_as_text(change),
        percent_change.abs(),
        price
    )
}

async fn get_and_notify(ticker: &str, percentage_threshold: f64) {
    // Check if we're in quiet mode
    if is_quiet_mode_enabled() && is_quiet_hours() {
        logger::debug(&format!("Quiet mode: skipping notification for {}", ticker));
        return;
    }

    let data = coin_gecko::get_ticker(ticker).await;

    if let Some(data) = data {
        if let Some(crypto_currency) = data.get(ticker) {
            let price = crypto_currency.usd.unwrap_or(0.0);
            let vol_24h = crypto_currency.usd_24h_vol.unwrap_or(0.0) / 1_000_000_000.0;
            let (price_change, percent_change) = get_price_change(ticker, price, percentage_threshold);
            logger::debug(&format!("price_change for {}: {:?}", ticker, price_change));

            if price_change != PriceChange::NoChange {
                let display_price = price;
                if ticker == "bitcoin" {
                    let (cbbi, fgi, bitdis) = tokio::join!(
                        cbbi::get_cbbi(),
                        alternative_me::get_fear_greed_index(),
                        bitdis::get_bitdis()
                    );

                    let text = format!(
                        "🟠 <b>Bitcoin</b> is {} {:.2}%! ${}\n📈 24h vol: ${:.2}B\n🔗 Reachable nodes: {}\n😈 F&GI: {}\n📊 CBBI: {}%",
                        price_change_as_text(&price_change),
                        percent_change.abs(),
                        display_price,
                        vol_24h,
                        bitdis,
                        fgi,
                        cbbi,
                    );
                    telegram::notify(ticker, &text).await;
                }

                let upper_case_ticker = to_upper_case(ticker);
                let text = format_price_notification(
                    &upper_case_ticker,
                    &price_change,
                    percent_change,
                    display_price
                );
                telegram::notify(ticker, &text).await;
            }
        }
    }

}


/// Runs a single iteration of price checking for all configured currencies.
async fn run_once() {
    let currencies = get_currencies();
    let mut set = tokio::task::JoinSet::new();

    for currency in currencies.iter() {
        let ticker = currency.ticker.clone();
        let percentage_threshold = currency.percentage_threshold;
        set.spawn(async move {
            get_and_notify(&ticker, percentage_threshold).await;
        });
    }

    set.join_all().await;
}
/// Main entry point for the notification service.
pub async fn run() {
    let currencies = get_currencies();
    logger::info(&format!("{} currencies defined.", currencies.len()));

    if is_quiet_mode_enabled() {
        let start = crate::config::get_quiet_mode_start_hour();
        let end = crate::config::get_quiet_mode_end_hour();
        logger::info(&format!("Quiet mode enabled: {} - {}", start, end));
    }

    let sleep_seconds = get_notifier_sleep() as u64;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(sleep_seconds));

    loop {
        run_once().await;
        interval.tick().await;
    }
}
