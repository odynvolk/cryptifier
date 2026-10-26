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
                logger::error(
                    format!("Failed to get chat IDs from Telegram: {}", describe_request_failure(&e)).as_str(),
                );
            }
        }
    }

    get_chat_ids_from_config().await
}

/// Renders a request failure without including the URL.
///
/// `reqwest::Error`'s `Display` appends `" for url ({url})"` — see
/// `reqwest-0.11.27/src/error.rs:205` — and both Telegram URLs are
/// `https://api.telegram.org/bot<api_key>/...`. So formatting one of these
/// errors into a log line writes the bot token to the log. Every Telegram error
/// path has to come through here rather than interpolating the error.
pub fn describe_request_failure(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "timed out".to_string()
    } else if let Some(status) = e.status() {
        // The branch a 429 takes, which used to count as success: `send_text`
        // discarded the response entirely.
        format!("HTTP {status}")
    } else if e.is_connect() {
        "could not connect".to_string()
    } else {
        // Not `{e}`: the message and the url suffix cannot be separated.
        "request failed".to_string()
    }
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
        .await?
        // Telegram answers a rejected send with a non-2xx status, and reqwest
        // does not treat status as an error by default. Without this, a 429 or a
        // 401 returned `Ok(())` and was counted as delivered — so the one line
        // an operator watches, `Notified N users`, went green on a message that
        // never arrived.
        .error_for_status()?;

    Ok(())
}

/// Upper bound on how many messages one notification attempts at once.
///
/// Sized to Telegram's roughly-30-messages-per-second-across-different-chats
/// guidance, so a full batch sits within the published tolerance. Well above any
/// hand-maintained `APP__TELEGRAM_CHAT_IDS`, but it has to exist: the
/// auto-discovery path adds IDs from `getUpdates` with no `limit`, so an idle bot
/// can accumulate a large set. Bounding the batch rather than throttling it also
/// keeps a cycle's duration constant, which is what the ~25 s budget in
/// `docker-compose.yml` assumes — a semaphore would make the drain grow with N
/// again.
pub const MAX_CONCURRENT_SENDS: usize = 30;

/// Trims a recipient list to `MAX_CONCURRENT_SENDS`, returning how many were left
/// out. Split out so the truncation is testable with no network.
pub fn select_recipients(chat_ids: Vec<String>) -> (Vec<String>, usize) {
    let skipped = chat_ids.len().saturating_sub(MAX_CONCURRENT_SENDS);
    let mut chat_ids = chat_ids;
    chat_ids.truncate(MAX_CONCURRENT_SENDS);
    (chat_ids, skipped)
}

/// Sends a notification to all configured Telegram chat IDs.
pub async fn notify(ticker: &str, text: &str) -> bool {
    let (chat_ids, skipped) = select_recipients(get_chat_ids().await);
    if skipped > 0 {
        logger::error(
            format!(
                "Skipping {} chat IDs, past the limit of {}. Check APP__TELEGRAM_GET_UPDATES.",
                skipped, MAX_CONCURRENT_SENDS
            )
            .as_str(),
        );
    }
    if chat_ids.is_empty() {
        return false;
    }

    // Deliberately concurrent. Awaiting each send one at a time made a cycle
    // cost 5 s per chat ID, so the drain window had to grow with a config value;
    // see the budget recorded in `docker-compose.yml`.
    let mut set = tokio::task::JoinSet::new();
    for chat_id in chat_ids {
        let text = text.to_string();
        // Carries the chat ID back so a failure can name which recipient did not
        // get it; without this a partial failure still logged as full success.
        set.spawn(async move { (chat_id.clone(), send_text(&chat_id, &text).await) });
    }

    // Every send is still awaited before returning, so a cycle never leaves work
    // running behind it — the concurrent part is only when they overlap.
    let mut success_count = 0;
    let mut failures: Vec<String> = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((_, Ok(()))) => success_count += 1,
            Ok((chat_id, Err(e))) => {
                failures.push(format!("{chat_id}: {}", describe_request_failure(&e)));
            }
            Err(e) => failures.push(format!("one send ended abnormally: {e}")),
        }
    }

    for failure in &failures {
        logger::error(format!("Telegram send failed: {failure}").as_str());
    }

    if success_count > 0 {
        logger::info(format!("Notified {} users about {}", success_count, ticker).as_str());
        true
    } else {
        logger::error("Failed to notify users of price change!");
        false
    }
}
