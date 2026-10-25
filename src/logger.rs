//! Logging configuration and utilities.
use chrono::Local;
use std::env;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Env var holding the level filter directives.
const LOG_LEVEL_VAR: &str = "APP__LOG_LEVEL";

/// Filter installed when `APP__LOG_LEVEL` is unset or is not valid directives.
const DEFAULT_LOG_LEVEL: &str = "info";

fn local_time_timer(buf: &mut tracing_subscriber::fmt::format::Writer) -> Result<(), std::fmt::Error> {
    let local = Local::now();
    buf.write_fmt(format_args!("{}", local.format("%H:%M:%S")))
}

/// Resolves the level filter from an arbitrary env lookup, so both branches are
/// testable without mutating the real process environment.
pub fn resolve_log_level(lookup: fn(&str) -> Result<String, env::VarError>) -> String {
    lookup(LOG_LEVEL_VAR)
        .ok()
        // Garbage must not be reported as if it were in effect — `init` would
        // fall back, so this resolves to the same thing it installs.
        .filter(|directives| tracing_subscriber::EnvFilter::try_new(directives).is_ok())
        .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_string())
}

/// Reads a var with the exact signature `resolve_log_level` expects.
fn lookup_var(key: &str) -> Result<String, env::VarError> {
    env::var(key)
}

/// The level filter actually in effect, for surfacing at startup.
pub fn effective_log_level() -> String {
    resolve_log_level(lookup_var)
}

pub fn init() {
    // Already validated by `resolve_log_level`, so `new` cannot panic here.
    let env_filter = tracing_subscriber::EnvFilter::new(effective_log_level());

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_timer(
            local_time_timer
                as for<'a, 'b> fn(&'a mut tracing_subscriber::fmt::format::Writer) -> Result<(), std::fmt::Error>,
        ))
        .with(env_filter)
        .init();
}

pub fn info(msg: &str) {
    tracing::info!(msg);
}

pub fn debug(msg: &str) {
    tracing::debug!(msg);
}

pub fn error(msg: &str) {
    tracing::error!(msg);
}
