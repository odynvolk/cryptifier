use cryptifier::common::PriceChange;
use cryptifier::notifier::format_price_notification;

#[test]
fn format_price_notification_reports_actual_rise() {
    let msg = format_price_notification("Ethereum", &PriceChange::Up, 5.0, 2000.0);
    assert!(msg.contains("Ethereum"));
    assert!(msg.contains("up"));
    assert!(msg.contains("5.00%"));
    assert!(msg.contains("$2000"));
}

#[test]
fn format_price_notification_down_reports_abs_magnitude() {
    let msg = format_price_notification("Ethereum", &PriceChange::Down, -3.5, 1500.0);
    assert!(msg.contains("Ethereum"));
    assert!(msg.contains("down"));
    assert!(msg.contains("3.50%"));
    assert!(msg.contains("$1500"));
}

#[test]
fn format_price_notification_no_change() {
    let msg = format_price_notification("Bitcoin", &PriceChange::NoChange, 0.0, 100000.0);
    assert!(msg.contains("no change"));
}

#[test]
fn bitcoin_detail_alert_suppresses_generic_alert() {
    // Bitcoin gets a rich detail alert (24h vol, nodes, F&G, CBBI) that
    // supersedes the generic price-move alert, so exactly one notification
    // is sent for bitcoin.
    assert!(cryptifier::notifier::suppress_generic_alert("bitcoin"));
}

#[test]
fn non_bitcoin_tickers_get_generic_alert() {
    assert!(!cryptifier::notifier::suppress_generic_alert("ethereum"));
    assert!(!cryptifier::notifier::suppress_generic_alert("solana"));
}
