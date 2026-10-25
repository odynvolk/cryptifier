//! Cooperative shutdown shared between the signal handlers and the run loop.
use std::time::Duration;
use tokio::sync::watch;

/// Shutdown request flag. Clones share one underlying state.
#[derive(Clone)]
pub struct Shutdown {
    tx: watch::Sender<bool>,
    rx: watch::Receiver<bool>,
}

impl Default for Shutdown {
    fn default() -> Self {
        let (tx, rx) = watch::channel(false);
        Self { tx, rx }
    }
}

impl Shutdown {
    /// A new shutdown flag that has not been requested yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask the run loop to stop at its next clean boundary.
    pub fn request(&self) {
        self.tx.send_replace(true);
    }

    /// Whether shutdown has been requested.
    pub fn is_requested(&self) -> bool {
        *self.rx.borrow()
    }

    /// Resolves as soon as shutdown has been requested.
    pub async fn wait(&self) {
        // A cloned receiver marks the current value as seen, so re-check it
        // each round instead of relying on `changed()` alone.
        let mut rx = self.rx.clone();
        while !self.is_requested() {
            if rx.changed().await.is_err() {
                break;
            }
        }
    }
}

/// Sleeps for `dur`, or until shutdown is requested.
/// Returns `true` when shutdown ended the wait early.
pub async fn sleep_or_shutdown(dur: Duration, shutdown: &Shutdown) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(dur) => false,
        _ = shutdown.wait() => true,
    }
}
