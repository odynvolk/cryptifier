//! Logging configuration and utilities.
use chrono::Local;
use std::env;
use std::str::FromStr;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Env var holding the level filter directives. Public so the startup report can
/// name it without repeating the literal.
pub const LOG_LEVEL_VAR: &str = "APP__LOG_LEVEL";

/// Filter installed when `APP__LOG_LEVEL` is unset, blank, or not valid
/// directives.
const DEFAULT_LOG_LEVEL: &str = "info";

fn local_time_timer(buf: &mut tracing_subscriber::fmt::format::Writer) -> Result<(), std::fmt::Error> {
    let local = Local::now();
    buf.write_fmt(format_args!("{}", local.format("%H:%M:%S")))
}

/// Outcome of resolving `APP__LOG_LEVEL`: the directives actually installed,
/// plus the configured text when it was set but could not be used.
///
/// Mirrors `config::NotifierSleep`. Without the second field a typo such as
/// `APP__LOG_LEVEL=deubg` is indistinguishable from an unset var, and the
/// startup line reports `info` as though it had been asked for — which is the
/// whole failure mode this PR set out to remove for the notifier sleep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLevel {
    /// Directives the filter was built from. Always parses and non-empty.
    pub directives: String,
    /// The rejected value, for surfacing at startup instead of being swallowed.
    pub rejected: Option<String>,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self {
            directives: DEFAULT_LOG_LEVEL.to_string(),
            rejected: None,
        }
    }
}

/// Whether installing these directives would actually let anything through.
///
/// Two separate failures, both of which end in a process that logs nothing:
/// one that does not parse, and one that parses as a *target* directive. A bare
/// word that is not a level name is read as a module path, which matches no
/// callsite here, so `EnvFilter::try_new` accepts it and the process then emits
/// nothing at all — measured at 0 lines against 9 for `info`. That is what makes
/// `deubg` worse than an ordinary typo: it is valid, so it is installed, echoed
/// back at startup, and silently ends all diagnostics.
///
/// Only bare words are checked. An explicit `target=level` is a deliberate
/// choice and passes untouched, so `tokio=trace` keeps working.
fn installs_useful_directives(directives: &str) -> bool {
    if tracing_subscriber::EnvFilter::try_new(directives).is_err() {
        return false;
    }

    directives.split(',').all(|part| {
        let part = part.trim();
        // `LevelFilter::from_str` accepts the empty string as `off`, so an empty
        // part has to be excluded separately or a trailing comma disables
        // logging through the very check meant to prevent it.
        part.contains('=') || (!part.is_empty() && LevelFilter::from_str(part).is_ok())
    })
}

/// Resolves the level filter from an arbitrary env lookup, so every branch is
/// testable without mutating the real process environment.
pub fn resolve_log_level(lookup: fn(&str) -> Result<String, env::VarError>) -> LogLevel {
    let configured = match lookup(LOG_LEVEL_VAR) {
        Ok(raw) => {
            // Trimmed once up front and used for everything after this point,
            // so the value validated, installed, and printed is one string.
            // Checking a trimmed copy while returning the padded original left
            // startup printing `Log level:  debug `.
            let raw = raw.trim().to_string();
            (!raw.is_empty()).then_some(raw)
        }
        Err(env::VarError::NotPresent) => None,
        // Set but unreadable as text is a config the operator got wrong, not an
        // absent one — same treatment as the notifier sleep. The placeholder
        // never parses, so the branches below already handle it.
        Err(env::VarError::NotUnicode(_)) => Some("<not valid UTF-8>".to_string()),
    };

    // Two reasons to reject a value here, neither of them safety: an invalid one
    // is not rejected by `init` at all (`EnvFilter::new` is `parse_lossy`), so a
    // different filter would be installed than the level reported at startup;
    // and an empty one parses to no directives, which makes `new` add its ERROR
    // default and silence every startup line — including the one reporting it.
    let usable = configured
        .as_deref()
        .filter(|directives| installs_useful_directives(directives));

    match usable {
        Some(directives) => LogLevel {
            directives: directives.to_string(),
            rejected: None,
        },
        // Report the configured text, never the fallback, or the operator sees
        // `info` twice and concludes the config arrived.
        None => LogLevel {
            directives: DEFAULT_LOG_LEVEL.to_string(),
            rejected: configured,
        },
    }
}

/// The level filter actually in effect, for surfacing at startup.
pub fn effective_log_level() -> LogLevel {
    // `env::var` cannot be passed directly: it is polymorphic over
    // `K: AsRef<OsStr>`, and a polymorphic fn item does not coerce to the
    // higher-ranked `for<'a> fn(&'a str)` this parameter needs. The closure
    // binds the lifetime at one call site, which is why it does.
    resolve_log_level(|key| env::var(key))
}

pub fn init() {
    // `new` cannot panic either way — it is `parse_lossy` — so the validation
    // above is what makes this filter match the level reported at startup.
    let env_filter = tracing_subscriber::EnvFilter::new(effective_log_level().directives);

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
