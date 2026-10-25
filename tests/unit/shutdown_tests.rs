//! Tests for the cooperative shutdown flag and the interruptible sleep.
use cryptifier::shutdown::{sleep_or_shutdown, Shutdown};
use std::time::{Duration, Instant};

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

#[tokio::test]
async fn sleep_or_shutdown_returns_at_once_when_already_requested() {
    let shutdown = Shutdown::new();
    shutdown.request();

    let started = Instant::now();
    let shutdown_won = sleep_or_shutdown(Duration::from_secs(60), &shutdown).await;

    assert!(shutdown_won);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "sleep_or_shutdown slept for {:?}",
        started.elapsed()
    );
}

#[tokio::test]
async fn sleep_or_shutdown_returns_while_sleeping() {
    let shutdown = Shutdown::new();
    let waiter = {
        let shutdown = shutdown.clone();
        tokio::spawn(async move { sleep_or_shutdown(Duration::from_secs(60), &shutdown).await })
    };

    // Let the waiter reach the sleep, then ask it to stop.
    tokio::time::sleep(Duration::from_millis(50)).await;
    shutdown.request();

    assert!(tokio::time::timeout(Duration::from_secs(1), waiter)
        .await
        .expect("waiter did not wake up on shutdown")
        .expect("waiter panicked"));
}

#[tokio::test]
async fn sleep_or_shutdown_waits_the_full_duration_without_shutdown() {
    let started = Instant::now();
    let shutdown_won = sleep_or_shutdown(Duration::from_millis(120), &Shutdown::new()).await;

    assert!(!shutdown_won);
    assert!(started.elapsed() >= Duration::from_millis(100));
}
