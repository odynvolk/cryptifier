//! Tests for the cooperative shutdown flag and the interruptible sleep.
//!
//! The timing cases run on tokio's paused virtual clock: time advances only when
//! a task is blocked on a timer, so the assertions are exact and the tests finish
//! instantly instead of spending real milliseconds waiting. `std::time::Instant`
//! would still measure the wall clock under `start_paused`, so these use
//! `tokio::time::Instant` to measure the time actually being asserted on.
use cryptifier::shutdown::{sleep_or_shutdown, Shutdown};
use std::time::Duration;

#[test]
fn shutdown_starts_unrequested() {
    assert!(!Shutdown::new().is_requested());
}

#[test]
fn request_is_visible_from_a_clone() {
    let shutdown = Shutdown::new();
    let clone = shutdown.clone();

    shutdown.request();

    assert!(shutdown.is_requested());
    assert!(clone.is_requested());
}

#[tokio::test(start_paused = true)]
async fn sleep_or_shutdown_returns_at_once_when_already_requested() {
    let shutdown = Shutdown::new();
    shutdown.request();

    let started = tokio::time::Instant::now();
    let shutdown_won = sleep_or_shutdown(Duration::from_secs(60), &shutdown).await;

    assert!(shutdown_won);
    // Exact rather than a margin: on a virtual clock, consuming none of a 60 s
    // sleep means none, and no scheduler stall can move it.
    assert_eq!(
        started.elapsed(),
        Duration::ZERO,
        "an already-requested shutdown still consumed virtual time"
    );
}

#[tokio::test(start_paused = true)]
async fn sleep_or_shutdown_returns_while_sleeping() {
    let shutdown = Shutdown::new();
    let waiter = {
        let shutdown = shutdown.clone();
        tokio::spawn(async move { sleep_or_shutdown(Duration::from_secs(60), &shutdown).await })
    };

    // Virtual time, so the waiter is deterministically inside the sleep before
    // the request lands. The old 50 ms real-time sleep only made it likelier.
    tokio::time::advance(Duration::from_millis(50)).await;
    shutdown.request();

    assert!(
        waiter.await.expect("waiter panicked"),
        "sleep_or_shutdown waited out the sleep instead of returning on shutdown"
    );
}

#[tokio::test(start_paused = true)]
async fn sleep_or_shutdown_waits_the_full_duration_without_shutdown() {
    let started = tokio::time::Instant::now();
    let shutdown_won = sleep_or_shutdown(Duration::from_millis(120), &Shutdown::new()).await;

    assert!(!shutdown_won);
    // Exact: the previous `>= 100ms` against a 120 ms sleep only proved it had
    // not returned early, and still cost real time to run.
    assert_eq!(started.elapsed(), Duration::from_millis(120));
}

#[tokio::test(start_paused = true)]
async fn wait_still_resolves_when_called_again_after_the_request() {
    let shutdown = Shutdown::new();
    shutdown.request();

    // Consume nothing: this first call resolves via the re-check in `wait()`.
    shutdown.wait().await;

    // Pins the observable behaviour, not the mechanism. It passes with and
    // without the re-check, because a clone inherits the initial cursor and so
    // still sees the request. Kept as the net for the refactor that *would*
    // make the re-check load-bearing — advancing a stored receiver, or using
    // `subscribe()`.
    //
    // Timeout in virtual time: a future that never resolves fails the assert
    // instead of hanging the runner.
    let second = tokio::time::timeout(Duration::from_secs(1), shutdown.wait()).await;
    assert!(
        second.is_ok(),
        "a second wait() blocked after the request had already been made"
    );
}
