use cryptifier::config::{get_currencies, get_notifier_sleep};

#[test]
fn config_currencies_default_to_bitcoin_and_ethereum() {
    let currencies = get_currencies();
    assert_eq!(currencies.len(), 2);
    assert_eq!(currencies[0].ticker, "bitcoin");
    assert_eq!(currencies[0].percentage_threshold, 2.0);
    assert_eq!(currencies[1].ticker, "ethereum");
    assert_eq!(currencies[1].percentage_threshold, 2.0);
}

#[test]
fn config_notifier_sleep_defaults_to_300() {
    assert_eq!(get_notifier_sleep(), 300);
}
