use once_cell::sync::Lazy;

/// Shared `reqwest::Client` so every API call reuses one connection pool
/// instead of building a fresh client (and pool) per request.
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .build()
        .expect("failed to build shared reqwest client")
});

/// Returns a reference to the shared reqwest client.
pub fn shared_client() -> &'static reqwest::Client {
    Lazy::force(&CLIENT)
}
