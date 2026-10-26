use cryptifier::config::{get_currencies, get_notifier_sleep, resolve_notifier_sleep};
use std::env::VarError;
use std::ffi::OsString;

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

// Each cycle makes external API calls and can send Telegram messages, so every
// rejected value has to land on 300 rather than a small clamp: `max(1)` would
// turn a typo into about four requests per second against public endpoints.

#[test]
fn config_notifier_sleep_unset_uses_300_without_complaining() {
    let resolved = resolve_notifier_sleep(|_| Err(VarError::NotPresent));
    assert_eq!(resolved.seconds, 300);
    // A warning here would fire on every normal container startup.
    assert_eq!(resolved.rejected, None);
}

#[test]
fn config_notifier_sleep_zero_falls_back_to_300() {
    // `tokio::time::sleep(Duration::ZERO)` is instantly ready, so a raw 0 would
    // spin the loop back-to-back with no delay at all.
    let resolved = resolve_notifier_sleep(|_| Ok("0".to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected, Some("0".to_string()));
}

#[test]
fn config_notifier_sleep_negative_falls_back_to_300() {
    // A negative value wraps via `as u64` to ~1.8e19, parking the loop forever.
    let resolved = resolve_notifier_sleep(|_| Ok("-5".to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected, Some("-5".to_string()));
}

#[test]
fn config_notifier_sleep_unparseable_falls_back_to_300() {
    let resolved = resolve_notifier_sleep(|_| Ok("five minutes".to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected, Some("five minutes".to_string()));
}

#[test]
fn config_notifier_sleep_blank_is_treated_as_unset() {
    // Same rule the log level uses: a key left with no value is not a value.
    let resolved = resolve_notifier_sleep(|_| Ok("   ".to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected, None);
}

#[test]
fn config_notifier_sleep_uses_a_valid_configured_value() {
    let resolved = resolve_notifier_sleep(|_| Ok("45".to_string()));
    assert_eq!(resolved.seconds, 45);
    assert_eq!(resolved.rejected, None);
}

#[test]
fn config_notifier_sleep_rejects_more_than_a_day() {
    // One stray zero (300 -> 30000000) was accepted before, which stopped
    // monitoring for about a year while startup printed the number as though it
    // had been asked for.
    let resolved = resolve_notifier_sleep(|_| Ok("30000000".to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected.as_deref(), Some("30000000"));
}

#[test]
fn config_notifier_sleep_rejects_i64_max_instead_of_passing_it_to_the_timer() {
    // The extreme of the same hole: it used to reach `Duration::from_secs`,
    // where the tokio timer panics rather than parking the task quietly.
    let resolved = resolve_notifier_sleep(|_| Ok(i64::MAX.to_string()));
    assert_eq!(resolved.seconds, 300);
    assert_eq!(resolved.rejected.as_deref(), Some(i64::MAX.to_string().as_str()));
}

#[test]
fn config_notifier_sleep_accepts_exactly_the_upper_bound() {
    let resolved = resolve_notifier_sleep(|_| Ok("86400".to_string()));
    assert_eq!(resolved.seconds, 86_400);
    assert_eq!(resolved.rejected, None);
}

#[test]
fn config_notifier_sleep_reports_a_value_that_is_not_valid_utf8() {
    // The key IS set here, just unreadable as text, so this is a config the
    // operator got wrong rather than one left alone.
    let resolved = resolve_notifier_sleep(|_| Err(VarError::NotUnicode(OsString::from("300"))));
    assert_eq!(resolved.seconds, 300);
    assert!(
        resolved.rejected.is_some(),
        "a set-but-unreadable value must still be reported, not defaulted silently"
    );
}
