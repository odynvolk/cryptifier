//! Main entry point for the Cryptifier application.
use cryptifier::logger;
use cryptifier::notifier;

fn main() {
    // Load environment variables from .env file (ignored if the file is absent).
    let env_path = std::env::current_dir().unwrap().join(".env");
    let _ = dotenvy::from_path(&env_path);

    logger::init();
    logger::info(&format!("Cryptifier starting..."));

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::select! {
            _ = notifier::run() => {
                logger::info(&format!("Notifier completed normally"));
            }
            _ = ctrl_c => {
                logger::info(&format!("Received SIGTERM (Ctrl+C), shutting down..."));
            }
        }
    });

    logger::info(&format!("Cryptifier stopped."));
}
