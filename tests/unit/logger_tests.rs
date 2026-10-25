//! Tests for log level resolution.
use cryptifier::logger::resolve_log_level;
use std::env::VarError;

#[test]
fn log_level_defaults_to_info_when_unset() {
    assert_eq!(resolve_log_level(|_| Err(VarError::NotPresent)), "info");
}

#[test]
fn log_level_uses_the_configured_value() {
    assert_eq!(resolve_log_level(|_| Ok("debug".to_string())), "debug");
}

#[test]
fn log_level_falls_back_when_the_configured_value_is_not_a_filter() {
    // `init` rejects these too, so reporting it would be a lie.
    assert_eq!(resolve_log_level(|_| Ok("=debug".to_string())), "info");
}
