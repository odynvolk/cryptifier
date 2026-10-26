use once_cell::sync::Lazy;
use std::time::Duration;

/// Backstop timeout for call sites that set no per-request `.timeout()` of their
/// own. Every current call site is tuned tighter (5s / 10s), so this only bites
/// on future code that forgets to.
///
/// Sized to the slowest deliberate request (the bitcoin sources' 10 s `join!`),
/// not left loose. That keeps a single untimed call inside the drain budget
/// derived in `docker-compose.yml`; at 30 s it alone exceeded the ~25 s a cycle
/// is allowed and would have put graceful shutdown back to exit 137.
///
/// Any new call site should still set its own `.timeout()` — this is a net, not
/// a tuning target. Re-check both against the budget if any timeout changes.
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

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
