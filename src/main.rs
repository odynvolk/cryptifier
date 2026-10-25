//! Main entry point for the Cryptifier application.
use cryptifier::config;
use cryptifier::logger;
use cryptifier::notifier;
use cryptifier::shutdown::Shutdown;
use std::path::Path;

fn main() {
    // Load .env if present. Whether it was found is reported once the logger is
    // up — a silently skipped file used to look exactly like a hang.
    let env_path = std::env::current_dir().unwrap().join(".env");
    let dotenv_result = dotenvy::from_path(&env_path);

    // Must stay after the dotenv load so APP__LOG_LEVEL from the file is honoured.
    logger::init();
    logger::info("Cryptifier starting...");
    log_startup_config(&env_path, &dotenv_result);

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

/// Reports the configuration that is genuinely in effect, so a config that did
/// not arrive is visible at a glance instead of inferred from later log lines.
fn log_startup_config(env_path: &Path, dotenv_result: &Result<(), dotenvy::Error>) {
    match dotenv_result {
        Ok(()) => logger::info(&format!("Loaded .env from {}", env_path.display())),
        // Deliberately INFO, not WARNING: the image ships only .env.example and
        // compose hands config over as real process env vars, so this is the
        // normal container startup and must not look like a fault.
        Err(err) if err.not_found() => logger::info(&format!(
            "No .env file at {}; using process environment",
            env_path.display()
        )),
        // A file that is there but unusable is the one case that is genuinely wrong.
        Err(err) => logger::error(&format!("Ignoring unreadable .env at {}: {err}", env_path.display())),
    }

    logger::info(&format!("Log level: {}", logger::effective_log_level()));
    logger::info(&format!("Notifier sleep: {}s", config::get_notifier_sleep()));
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
