//! Tests for Telegram recipient selection and error rendering.
use cryptifier::notifiers::telegram::{describe_request_failure, select_recipients, MAX_CONCURRENT_SENDS};
use std::time::Duration;

fn ids(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("{i}")).collect()
}

#[test]
fn telegram_select_recipients_passes_a_small_batch_through() {
    let (chosen, skipped) = select_recipients(ids(3));
    assert_eq!(chosen.len(), 3);
    assert_eq!(skipped, 0);
}

#[test]
fn telegram_select_recipients_keeps_exactly_the_limit() {
    let (chosen, skipped) = select_recipients(ids(MAX_CONCURRENT_SENDS));
    assert_eq!(chosen.len(), MAX_CONCURRENT_SENDS);
    assert_eq!(skipped, 0);
}

#[test]
fn telegram_select_recipients_truncates_and_reports_the_surplus() {
    let (chosen, skipped) = select_recipients(ids(MAX_CONCURRENT_SENDS + 5));
    assert_eq!(chosen.len(), MAX_CONCURRENT_SENDS);
    // Reported rather than dropped quietly: the surplus is undelivered alerts.
    assert_eq!(skipped, 5);
    assert_eq!(chosen.first().map(String::as_str), Some("0"));
}

#[test]
fn telegram_select_recipients_handles_an_empty_list() {
    let (chosen, skipped) = select_recipients(vec![]);
    assert!(chosen.is_empty());
    assert_eq!(skipped, 0);
}

#[tokio::test]
async fn telegram_request_failure_description_never_leaks_the_bot_token() {
    const FAKE_TOKEN: &str = "123456:SECRET-does-not-exist";
    // Loopback on a closed port: refuses immediately, reaches nothing external.
    let url = format!("https://127.0.0.1:1/bot{FAKE_TOKEN}/getUpdates");
    let err = reqwest::Client::new()
        .get(&url)
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .expect_err("127.0.0.1:1 should refuse the connection");

    // The premise, asserted so this test fails loudly if reqwest ever stops
    // appending the url: the raw error really does carry the token.
    let raw = format!("{err}");
    assert!(
        raw.contains(FAKE_TOKEN),
        "premise changed: reqwest no longer renders the url, so re-check why this exists"
    );

    let described = describe_request_failure(&err);
    assert!(
        !described.contains(FAKE_TOKEN),
        "bot token leaked into a log message: {described}"
    );
}
