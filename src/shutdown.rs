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
    ///
    /// `clone()` copies the parent's version cursor (`watch.rs`: `let version =
    /// self.version`), so a child is *not* marked as having seen the current
    /// value — and a fresh receiver already considers the initial value seen, so
    /// `changed()` blocks until an actual send rather than completing at once.
    /// Since `self.rx` is only ever cloned and never advanced, every child keeps
    /// the initial cursor and so observes any `request()`.
    ///
    /// That is why the re-check below is redundant today, and why removing it
    /// breaks no test. It is kept because it does not depend on that chain: it
    /// resolves an already-requested flag without touching the watch at all.
    /// Advancing a stored `rx`, or switching to `tx.subscribe()` — which *does*
    /// mark the current value as seen — would leave `changed()` unable to see a
    /// request that landed earlier, and a `wait()` that never resolves means
    /// SIGTERM goes unnoticed and the container is killed at the grace period.
    pub async fn wait(&self) {
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
