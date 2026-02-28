use crate::cache::SHORT_CACHE;
use crate::config::CONFIG;
use crate::logger;
use std::pin::Pin;
use std::future::Future;

async fn get_chat_ids_from_config() -> Vec<String> {
    CONFIG
        .telegram_chat_ids
        .as_ref()
        .map(|ids| ids.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

async fn get_chat_ids() -> Vec<String> {
    if CONFIG.telegram_get_updates.unwrap_or(false) {
        let value = SHORT_CACHE.get("getUpdates");
        if let Some(value) = value {
            return serde_json::from_str(&value).unwrap_or_default();
        }

        let client = reqwest::Client::new();
        let api_key = CONFIG.telegram_api_key.clone().unwrap_or_default();

        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates",
            api_key
        );

        let response = client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await;

        match response {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let mut chat_ids = get_chat_ids_from_config().await;

                        if let Some(serde_json::Value::Array(result_array)) = json.get("result") {
                            for update in result_array {
                                if let Some(message) = update.get("message") {
                                    if let Some(chat) = message.get("chat") {
                                        if let Some(id) = chat.get("id") {
                                            if let Some(id_str) = id.as_i64() {
                                                chat_ids.push(id_str.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let result: Vec<String> = chat_ids.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();

                        logger::debug(format!("Got chatIds from Telegram: {}", serde_json::to_string(&result).unwrap_or_default()).as_str());

                        SHORT_CACHE.set("getUpdates", serde_json::to_string(&result).unwrap_or_default());

                        return result;
                    }
                    Err(e) => {
                        logger::error(format!("Failed to parse getUpdates response: {}", e).as_str());
                    }
                }
            }
            Err(e) => {
                logger::error(format!("Failed to get chat IDs from Telegram: {}", e).as_str());
            }
        }
    }

    get_chat_ids_from_config().await
}

async fn send_text(chat_id: &str, text: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let api_key = CONFIG.telegram_api_key.clone().unwrap_or_default();

    let url = format!("https://api.telegram.org/bot{}/sendMessage", api_key);

    let body = serde_json::json!({
        "chat_id": chat_id,
        "parse_mode": "html",
        "text": text,
    });

    client
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    Ok(())
}

pub async fn notify(ticker: &str, text: &str) -> bool {
    match get_chat_ids().await {
        chat_ids if chat_ids.is_empty() => false,
        chat_ids => {
            let texts_to_send: Vec<_> = chat_ids
                .iter()
                .map(|chat_id| Box::pin(send_text(chat_id, text)) as Pin<Box<dyn Future<Output = Result<(), reqwest::Error>> + Send>>)
                .collect();

            let mut success_count = 0;
            for future in texts_to_send {
                if future.await.is_ok() {
                    success_count += 1;
                }
            }

            if success_count > 0 {
                logger::info(format!("Notified {} users about {}", success_count, ticker).as_str());
                true
            } else {
                logger::error("Failed to notify users of price change!");
                false
            }
        }
    }
}
