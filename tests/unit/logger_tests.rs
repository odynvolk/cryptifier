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
    // `new` would install an ERROR-only filter for this, so reporting it
    // verbatim would be a lie about what is in effect.
    assert_eq!(resolve_log_level(|_| Ok("=debug".to_string())), "info");
}

#[test]
fn log_level_treats_an_empty_value_as_unset() {
    // `EnvFilter::new("")` parses to no directives and installs an ERROR-only
    // filter — every startup line, including the one reporting this value,
    // would be suppressed.
    assert_eq!(resolve_log_level(|_| Ok(String::new())), "info");
}

#[test]
fn log_level_treats_a_whitespace_only_value_as_unset() {
    assert_eq!(resolve_log_level(|_| Ok("   ".to_string())), "info");
}
