//! Main entry point for the Cryptifier application.
use cryptifier::logger;
use cryptifier::notifier;
use cryptifier::shutdown::Shutdown;

fn main() {
    // Load environment variables from .env file (ignored if the file is absent).
    let env_path = std::env::current_dir().unwrap().join(".env");
    let _ = dotenvy::from_path(&env_path);

    logger::init();
    logger::info("Cryptifier starting...");

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let shutdown = Shutdown::new();
        let signal_shutdown = shutdown.clone();

        // A signal only *requests* shutdown; the run loop decides when to stop.
        // Nothing is cancelled here, so in-flight work drains on its own.
        tokio::join!(
            async move {
                let signal = wait_for_signal().await;
                logger::info(&format!("Received {signal}, shutting down..."));
                signal_shutdown.request();
            },
            notifier::run(shutdown),
        );
    });

    logger::info("Cryptifier stopped.");
}

/// Waits for the first shutdown signal and returns its name, so the log says
/// what actually fired rather than a guess.
#[cfg(unix)]
async fn wait_for_signal() -> &'static str {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut terminate) => tokio::select! {
            _ = tokio::signal::ctrl_c() => "SIGINT (Ctrl+C)",
            _ = terminate.recv() => "SIGTERM",
        },
        // Cannot observe SIGTERM on this host; Ctrl+C is all we have.
        Err(_) => wait_for_ctrl_c().await,
    }
}

/// `tokio::signal::unix` does not exist on Windows, so Ctrl+C is the only signal.
#[cfg(not(unix))]
async fn wait_for_signal() -> &'static str {
    wait_for_ctrl_c().await
}

async fn wait_for_ctrl_c() -> &'static str {
    // Errors only if the process-wide handler cannot be installed; keep running.
    let _ = tokio::signal::ctrl_c().await;
    "SIGINT (Ctrl+C)"
}
