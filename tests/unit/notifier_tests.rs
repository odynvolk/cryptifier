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
