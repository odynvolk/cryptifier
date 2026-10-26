//! Application configuration management.
use crate::logger;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrencyConfig {
    pub ticker: String,
    /// Percentage threshold for price change notification (e.g., 5.0 = 5%)
    pub percentage_threshold: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    /// Resolved, never the raw config: the loop and the startup log must read the
    /// same number an unusable value was corrected to.
    pub notifier_sleep: NotifierSleep,
    pub telegram_api_key: Option<String>,
    pub telegram_chat_ids: Option<String>,
    pub telegram_get_updates: Option<bool>,
    pub currencies: Option<Vec<CurrencyConfig>>,
    pub quiet_mode_enabled: Option<bool>,
    pub quiet_mode_start_hour: Option<i64>,
    pub quiet_mode_end_hour: Option<i64>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notifier_sleep: NotifierSleep::default(),
            telegram_api_key: None,
            telegram_chat_ids: None,
            telegram_get_updates: Some(false),
            currencies: None,
            quiet_mode_enabled: Some(false),
            quiet_mode_start_hour: Some(0),
            quiet_mode_end_hour: Some(6),
        }
    }
}

/// Env var holding the seconds to sleep between check cycles. Public so the startup
/// report can name it without repeating the literal.
pub const NOTIFIER_SLEEP_VAR: &str = "APP__NOTIFIER_SLEEP";

/// Seconds slept when `APP__NOTIFIER_SLEEP` is unset or unusable.
const DEFAULT_NOTIFIER_SLEEP: i64 = 300;

/// Smallest usable interval. Zero is no delay at all, which would poll back to
/// back.
pub const MIN_NOTIFIER_SLEEP: i64 = 1;

/// Largest usable interval — one day. A background price watcher has no reason
/// to wait longer, and a stray zero (300 -> 30000000) would otherwise stop
/// monitoring for about a year while the startup log printed the number as if it
/// were intended. At the extreme it reaches `Duration::from_secs` and the tokio
/// timer, which panics rather than parking: `Sleep::poll` is
/// `Err(e) => panic!("timer error: {e}")` for "timer duration exceeds maximum
/// duration", and tokio's own `far_future()` is only ~30 years because further
/// out "overflows on macOS".
pub const MAX_NOTIFIER_SLEEP: i64 = 86_400;

/// Outcome of resolving `APP__NOTIFIER_SLEEP`: the seconds actually slept, plus
/// the configured text when it was set but could not be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotifierSleep {
    /// Seconds the loop sleeps after each cycle. Always positive.
    pub seconds: i64,
    /// The rejected value, for surfacing at startup instead of being swallowed.
    pub rejected: Option<String>,
}

impl Default for NotifierSleep {
    fn default() -> Self {
        Self {
            seconds: DEFAULT_NOTIFIER_SLEEP,
            rejected: None,
        }
    }
}

/// Resolves the sleep from an arbitrary env lookup, so every fallback branch is
/// testable without mutating process env (`set_var` is disallowed by
/// `.clippy.toml`).
///
/// Anything outside `MIN_NOTIFIER_SLEEP..=MAX_NOTIFIER_SLEEP`, or unparseable,
/// falls back to `DEFAULT_NOTIFIER_SLEEP` rather than being clamped to a bound.
/// A 0 sleep is no delay at all, a negative one wraps through `as u64` and parks
/// the loop forever, and every cycle makes four external GETs plus a Telegram
/// send — so clamping to `1` would turn a typo into a fast poll of public
/// endpoints. The upper end is rejected for the same reason in the other
/// direction: degrade to something sane, not merely to something in range.
pub fn resolve_notifier_sleep(lookup: fn(&str) -> Result<String, env::VarError>) -> NotifierSleep {
    let configured = match lookup(NOTIFIER_SLEEP_VAR) {
        Ok(raw) => {
            let raw = raw.trim().to_string();
            // A key left with no value is not a value — same rule the log level uses.
            (!raw.is_empty()).then_some(raw)
        }
        Err(env::VarError::NotPresent) => None,
        // Set but not readable as text is still a config the operator got wrong.
        // Folding it into `NotPresent` would leave `rejected` empty and swallow
        // it, which is the opposite of what this type exists for. The
        // placeholder never parses, so the branches below already handle it.
        Err(env::VarError::NotUnicode(_)) => Some("<not valid UTF-8>".to_string()),
    };

    let usable = configured
        .as_deref()
        .and_then(|raw| raw.parse::<i64>().ok())
        .filter(|seconds| (MIN_NOTIFIER_SLEEP..=MAX_NOTIFIER_SLEEP).contains(seconds));

    match usable {
        Some(seconds) => NotifierSleep {
            seconds,
            rejected: None,
        },
        // Report the configured text, never the fallback, or the operator sees
        // 300 twice and concludes the config arrived.
        None => NotifierSleep {
            seconds: DEFAULT_NOTIFIER_SLEEP,
            rejected: configured,
        },
    }
}

/// The notifier sleep actually in effect, for the loop and the startup log.
pub fn effective_notifier_sleep() -> NotifierSleep {
    CONFIG.notifier_sleep.clone()
}

/// Global application configuration.
pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let currencies_str = env::var("APP__CURRENCIES").unwrap_or_else(|_| {
        "[{\"ticker\": \"bitcoin\", \"percentage_threshold\": 2.0}, {\"ticker\": \"ethereum\", \"percentage_threshold\": 2.0}]".to_string()
    });
    let quiet_mode_enabled_str = env::var("APP__QUIET_MODE_ENABLED").unwrap_or_else(|_| "false".to_string());
    let quiet_mode_start_hour_str = env::var("APP__QUIET_MODE_START_HOUR").unwrap_or_else(|_| "0".to_string());
    let quiet_mode_end_hour_str = env::var("APP__QUIET_MODE_END_HOUR").unwrap_or_else(|_| "6".to_string());
    let currencies: Vec<CurrencyConfig> = serde_json::from_str(&currencies_str).unwrap_or_else(|_| {
        vec![
            CurrencyConfig {
                ticker: "bitcoin".to_string(),
                percentage_threshold: 5.0,
            },
            CurrencyConfig {
                ticker: "ethereum".to_string(),
                percentage_threshold: 5.0,
            },
        ]
    });

    logger::debug(&format!("Loaded currencies: {:?}", currencies));
    AppConfig {
        // `env::var` cannot be passed directly — it is polymorphic over
        // `K: AsRef<OsStr>` and will not coerce to this higher-ranked parameter.
        notifier_sleep: resolve_notifier_sleep(|key| env::var(key)),
        telegram_api_key: env::var("APP__TELEGRAM_API_KEY").ok(),
        telegram_chat_ids: env::var("APP__TELEGRAM_CHAT_IDS").ok(),
        telegram_get_updates: env::var("APP__TELEGRAM_GET_UPDATES").ok().and_then(|s| s.parse().ok()),
        currencies: Some(currencies),
        quiet_mode_enabled: quiet_mode_enabled_str.parse().ok(),
        quiet_mode_start_hour: quiet_mode_start_hour_str.parse().ok(),
        quiet_mode_end_hour: quiet_mode_end_hour_str.parse().ok(),
    }
});

/// Returns the list of currencies configured for monitoring.
pub fn get_currencies() -> Vec<CurrencyConfig> {
    CONFIG.currencies.clone().unwrap_or_default()
}

/// Returns the notifier sleep interval in seconds, always positive.
pub fn get_notifier_sleep() -> i64 {
    CONFIG.notifier_sleep.seconds
}

/// Returns whether quiet mode is enabled.
pub fn is_quiet_mode_enabled() -> bool {
    CONFIG.quiet_mode_enabled.unwrap_or(false)
}

/// Returns the quiet mode start hour (0-23).
pub fn get_quiet_mode_start_hour() -> i64 {
    CONFIG.quiet_mode_start_hour.unwrap_or(0)
}

/// Returns the quiet mode end hour (0-23).
pub fn get_quiet_mode_end_hour() -> i64 {
    CONFIG.quiet_mode_end_hour.unwrap_or(6)
}

/// Check if current time is within quiet mode hours.
pub fn is_quiet_hours() -> bool {
    use chrono::{Local, Timelike};

    let now = Local::now();
    let current_hour = now.hour() as u64;
    let start_hour = get_quiet_mode_start_hour() as u64;
    let end_hour = get_quiet_mode_end_hour() as u64;

    if start_hour < end_hour {
        // Normal case: e.g., 00:00 to 06:00
        current_hour >= start_hour && current_hour < end_hour
    } else if start_hour > end_hour {
        // Wrap around midnight: e.g., 22:00 to 06:00
        current_hour >= start_hour || current_hour < end_hour
    } else {
        // Same hour means entire day
        true
    }
}
