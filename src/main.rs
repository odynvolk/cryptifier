//! Main entry point for the Cryptifier application.
use cryptifier::config;
use cryptifier::logger;
use cryptifier::notifier;
use cryptifier::shutdown::Shutdown;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// Outcome of locating and loading the optional `.env`: `None` when no path could
/// even be formed, otherwise the path tried and what loading it returned.
type DotenvOutcome = Option<(PathBuf, Result<(), dotenvy::Error>)>;

fn main() {
    // Load .env if present. Whether it was found is reported once the logger is
    // up — a silently skipped file used to look exactly like a hang.
    let dotenv_outcome = resolve_env_path(std::env::current_dir()).map(load_dotenv);

    // Must stay after the dotenv load so APP__LOG_LEVEL from the file is honoured.
    logger::init();
    logger::info("Cryptifier starting...");
    log_startup_config(&dotenv_outcome);

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

/// Loads `path`, keeping it alongside the result so the caller can report where
/// it looked.
fn load_dotenv(path: PathBuf) -> (PathBuf, Result<(), dotenvy::Error>) {
    let result = dotenvy::from_path(&path);
    (path, result)
}

/// Forms the `.env` path beside the working directory, or `None` if there is no
/// readable working directory.
///
/// The old `current_dir().unwrap()` panicked here, aborting before any log line
/// could explain why. Losing the working directory is not fatal: it only means no
/// relative `.env` can be located, so the process environment stands on its own
/// — which is already how the container is configured.
fn resolve_env_path(cwd: io::Result<PathBuf>) -> Option<PathBuf> {
    cwd.ok().map(|dir| dir.join(".env"))
}

/// Reports the configuration that is genuinely in effect, so a config that did
/// not arrive is visible at a glance instead of inferred from later log lines.
fn log_startup_config(dotenv_outcome: &DotenvOutcome) {
    match dotenv_outcome {
        Some((path, Ok(()))) => logger::info(&format!("Loaded .env from {}", path.display())),
        // Deliberately INFO, not WARNING: the image ships only .env.example and
        // compose hands config over as real process env vars, so this is the
        // normal container startup and must not look like a fault.
        Some((path, Err(err))) if err.not_found() => logger::info(&format!(
            "No .env file at {}; using process environment",
            path.display()
        )),
        // A file that is there but unusable is the one case that is genuinely wrong.
        Some((path, Err(err))) => logger::error(&format!("Ignoring unreadable .env at {}: {err}", path.display())),
        // Also INFO: with no working directory there is nothing to have missed,
        // and the process environment is a complete configuration on its own.
        None => logger::info("Working directory unavailable; using process environment"),
    }

    // A value that was set but could not be used has to be named, or a typo
    // reads as an intentional default. This one matters more than the others:
    // a wrong level is what makes every other startup line disappear, so the
    // report is emitted at ERROR against the filter actually installed.
    let log_level = logger::effective_log_level();
    if let Some(configured) = &log_level.rejected {
        logger::error(&format!(
            "Ignoring {}='{}': not a log level (off, error, warn, info, debug, trace) and not a `target=level` directive. Using '{}'.",
            logger::LOG_LEVEL_VAR,
            configured,
            log_level.directives
        ));
    }
    logger::info(&format!("Log level: {}", log_level.directives));

    // The loop sleeps this same resolved value, so the number printed here is
    // the number in effect even when the configured one was discarded.
    let notifier_sleep = config::effective_notifier_sleep();
    if let Some(configured) = &notifier_sleep.rejected {
        // ERROR, not INFO: the loop would otherwise have spun with no delay at
        // all, never woken again, or slept for about a year — and a recovered
        // typo is still a typo. Keeps running, because a wrong interval is not a
        // reason to stop monitoring.
        logger::error(&format!(
            "Ignoring {}='{}': not a whole number of seconds within {}..{}. Using {}s.",
            config::NOTIFIER_SLEEP_VAR,
            configured,
            config::MIN_NOTIFIER_SLEEP,
            config::MAX_NOTIFIER_SLEEP,
            notifier_sleep.seconds
        ));
    }
    logger::info(&format!("Notifier sleep: {}s", notifier_sleep.seconds));
}

/// Waits for the first shutdown signal and returns its name, so the log says
/// what actually fired rather than a guess.
#[cfg(unix)]
async fn wait_for_signal() -> &'static str {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        // Routed through the same helper as the fallback, so an uninstallable
        // Ctrl+C handler cannot complete this arm and trigger a false shutdown.
        Ok(mut terminate) => tokio::select! {
            _ = wait_for_ctrl_c() => "SIGINT (Ctrl+C)",
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
    // An Err means the process-wide handler could not be installed, not that a
    // signal arrived. Returning here would be read as a real signal and shut the
    // service down at startup, so back off and keep waiting instead.
    loop {
        match tokio::signal::ctrl_c().await {
            Ok(()) => return "SIGINT (Ctrl+C)",
            Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_env_path;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn appends_env_to_a_readable_working_directory() {
        let path = resolve_env_path(Ok(PathBuf::from("/srv/app")));
        assert_eq!(path, Some(PathBuf::from("/srv/app/.env")));
    }

    /// The case that used to `unwrap()` and abort the process before any log
    /// line could be emitted. No path must mean "fall back to process env",
    /// and must not panic — reintroducing `unwrap()` fails this test.
    #[test]
    fn yields_no_path_when_the_working_directory_is_unreadable() {
        let err = io::Error::new(io::ErrorKind::NotFound, "working directory removed");
        assert_eq!(resolve_env_path(Err(err)), None);
    }
}
