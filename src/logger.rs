//! Logging configuration and utilities.
use chrono::Local;
use tracing_subscriber::{fmt, prelude::*};

fn local_time_timer(buf: &mut tracing_subscriber::fmt::format::Writer) -> Result<(), std::fmt::Error> {
    let local = Local::now();
    buf.write_fmt(format_args!("{}", local.format("%H:%M:%S")))
}

pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_timer(
            local_time_timer
                as for<'a, 'b> fn(&'a mut tracing_subscriber::fmt::format::Writer) -> Result<(), std::fmt::Error>,
        ))
        .with(env_filter)
        .init();
}

pub fn info(msg: &str) {
    tracing::info!("{}", msg);
    println!("{}", msg);
}

pub fn debug(msg: &str) {
    tracing::debug!("{}", msg);
    println!("{}", msg);
}

pub fn error(msg: &str) {
    tracing::error!("{}", msg);
}
