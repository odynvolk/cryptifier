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
        }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = {
        let currencies_str = env::var("APP__CURRENCIES").unwrap_or_else(|_| "[{\"ticker\": \"bitcoin\", \"increment\": 10}, {\"ticker\": \"ethereum\", \"increment\": 100}]".to_string());
        let currencies: Vec<CurrencyConfig> = serde_json::from_str(&currencies_str)
            .unwrap_or_else(|_| vec![
                CurrencyConfig {
                    ticker: "bitcoin".to_string(),
                    increment: 1000,
                },
                CurrencyConfig {
                    ticker: "ethereum".to_string(),
                    increment: 100,
                },
            ]);

        println!("{}", currencies_str);
        AppConfig {
            log_level: env::var("APP__LOG_LEVEL").ok(),
            notifier_sleep: env::var("APP__NOTIFIER_SLEEP")
                .ok()
                .and_then(|s| s.parse().ok()),
            telegram_api_key: env::var("APP__TELEGRAM_API_KEY").ok(),
            telegram_chat_ids: env::var("APP__TELEGRAM_CHAT_IDS").ok(),
            telegram_get_updates: env::var("APP__TELEGRAM_GET_UPDATES")
                .ok()
                .and_then(|s| s.parse().ok()),
            currencies: Some(currencies),
        }
    };
}

pub fn get_currencies() -> Vec<CurrencyConfig> {
    CONFIG.currencies.clone().unwrap_or_default()
}

pub fn get_notifier_sleep() -> i64 {
    CONFIG.notifier_sleep.unwrap_or(300)
}
