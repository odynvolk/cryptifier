//! Main entry point for the Cryptifier application.
use cryptifier::logger;
use cryptifier::notifier;

fn main() {
    // Load environment variables from .env file
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

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::select! {
            _ = notifier::run() => {
                logger::info("Notifier completed normally");
            }
            _ = ctrl_c => {
                logger::info("Received SIGTERM (Ctrl+C), shutting down...");
            }
        }
    });

    logger::info("Cryptifier stopped.");
}
