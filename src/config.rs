//! Application configuration management.
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrencyConfig {
    pub ticker: String,
    pub increment: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub log_level: Option<String>,
    pub notifier_sleep: Option<i64>,
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
            log_level: Some("info".to_string()),
            notifier_sleep: Some(300),
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

/// Global application configuration.
pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    let currencies_str = env::var("APP__CURRENCIES").unwrap_or_else(|_| {
        "[{\"ticker\": \"bitcoin\", \"increment\": 10}, {\"ticker\": \"ethereum\", \"increment\": 100}]".to_string()
    });
    let quiet_mode_enabled_str = env::var("APP__QUIET_MODE_ENABLED").unwrap_or_else(|_| "false".to_string());
    let quiet_mode_start_hour_str = env::var("APP__QUIET_MODE_START_HOUR").unwrap_or_else(|_| "0".to_string());
    let quiet_mode_end_hour_str = env::var("APP__QUIET_MODE_END_HOUR").unwrap_or_else(|_| "6".to_string());
    let currencies: Vec<CurrencyConfig> = serde_json::from_str(&currencies_str).unwrap_or_else(|_| {
        vec![
            CurrencyConfig {
                ticker: "bitcoin".to_string(),
                increment: 1000,
            },
            CurrencyConfig {
                ticker: "ethereum".to_string(),
                increment: 100,
            },
        ]
    });

    println!("{}", currencies_str);
    AppConfig {
        log_level: env::var("APP__LOG_LEVEL").ok(),
        notifier_sleep: env::var("APP__NOTIFIER_SLEEP").ok().and_then(|s| s.parse().ok()),
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

/// Returns the notifier sleep interval in seconds.
pub fn get_notifier_sleep() -> i64 {
    CONFIG.notifier_sleep.unwrap_or(300)
}

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
