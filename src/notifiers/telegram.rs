//! Telegram notification service implementation.
use crate::cache::SHORT_CACHE;
use crate::config::CONFIG;
use crate::logger;

/// Retrieves chat IDs from the application configuration.
use crate::http::shared_client;
async fn get_chat_ids_from_config() -> Vec<String> {
    let configured = CONFIG
        .telegram_chat_ids
        .as_ref()
        .map(|ids| {
            ids.split(',')
                .map(|id| id.trim())
                .filter(|id| !id.is_empty())
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Duplicated IDs would now go out concurrently, which risks Telegram's
    // roughly-one-message-per-second-per-chat limit rather than merely wasting a
    // slow retry the way the old sequential loop did.
    let mut seen = std::collections::HashSet::new();
    configured.into_iter().filter(|id| seen.insert(id.clone())).collect()
}

/// Retrieves chat IDs from configuration or auto-discovers them from Telegram.
async fn get_chat_ids() -> Vec<String> {
    if CONFIG.telegram_get_updates.unwrap_or(false) {
        let value = SHORT_CACHE.get("getUpdates");
        if let Some(value) = value {
            return serde_json::from_str(&value).unwrap_or_default();
        }

        let client = shared_client();
        let api_key = CONFIG.telegram_api_key.clone().unwrap_or_default();

        let url = format!("https://api.telegram.org/bot{}/getUpdates", api_key);

        let response = client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await;

        match response {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
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

                    let result: Vec<String> = chat_ids
                        .into_iter()
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();

                    logger::debug(
                        format!(
                            "Got chatIds from Telegram: {}",
                            serde_json::to_string(&result).unwrap_or_default()
                        )
                        .as_str(),
                    );

                    SHORT_CACHE.set("getUpdates", serde_json::to_string(&result).unwrap_or_default());

                    return result;
                }
                Err(e) => {
                    logger::error(format!("Failed to parse getUpdates response: {}", e).as_str());
                }
            },
            Err(e) => {
                logger::error(format!("Failed to get chat IDs from Telegram: {}", e).as_str());
            }
        }
    }

    get_chat_ids_from_config().await
}

/// Sends a text message to a specific Telegram chat.
async fn send_text(chat_id: &str, text: &str) -> Result<(), reqwest::Error> {
    let client = shared_client();
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

/// Sends a notification to all configured Telegram chat IDs.
pub async fn notify(ticker: &str, text: &str) -> bool {
    let chat_ids = get_chat_ids().await;
    if chat_ids.is_empty() {
        return false;
    }

    // Deliberately concurrent. Awaiting each send one at a time made a cycle
    // cost 5 s per chat ID, so the drain window had to grow with a config value;
    // see the budget recorded in `docker-compose.yml`.
    let mut set = tokio::task::JoinSet::new();
    for chat_id in chat_ids {
        let text = text.to_string();
        set.spawn(async move { send_text(&chat_id, &text).await });
    }

    // Every send is still awaited before returning, so a cycle never leaves work
    // running behind it — the concurrent part is only when they overlap.
    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        if matches!(&result, Ok(Ok(()))) {
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
