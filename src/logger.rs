use tracing_subscriber::{fmt, prelude::*};

pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
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
