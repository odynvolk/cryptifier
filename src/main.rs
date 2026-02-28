use cryptifier::logger;
use cryptifier::notifier;

fn main() {
    let env_path = std::env::current_dir().unwrap().join(".env");

    if let Ok(content) = std::fs::read_to_string(&env_path) {
        for line in content.lines() {
            if !line.is_empty() && !line.starts_with('#') {
                if let Some((key, value)) = line.split_once('=') {
                    std::env::set_var(key, value);
                }
            }
        }
    }

    logger::init();
    logger::info("Cryptifier starting...");

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            notifier::run().await;
        });
}
