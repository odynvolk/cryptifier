use once_cell::sync::Lazy;
use std::time::Duration;

/// Backstop timeout for call sites that set no per-request `.timeout()` of their
/// own. Every current call site is tuned tighter (5s / 10s), so this only bites
/// on future code that forgets to. Kept loose on purpose so it never silently
/// tightens a deliberately tuned request, and still far below the 300s default
/// notifier interval so one stuck call cannot stall a whole cycle.
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Fail fast on an unreachable host instead of burning the whole request budget
/// waiting for a TCP connection. This container is capped at 0.25 CPUs / 64 MB,
/// where a handshake attempt against a dead endpoint is the expensive case.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Shared `reqwest::Client` so every API call reuses one connection pool
/// instead of building a fresh client (and pool) per request.
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(DEFAULT_REQUEST_TIMEOUT)
        .build()
        .expect("failed to build shared reqwest client")
});

/// Returns a reference to the shared reqwest client.
pub fn shared_client() -> &'static reqwest::Client {
    Lazy::force(&CLIENT)
}
