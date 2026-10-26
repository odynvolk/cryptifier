//! Tests for log level resolution.
use cryptifier::logger::resolve_log_level;
use std::env::VarError;
use std::ffi::OsString;

/// Unwrap the resolved directives, failing with the rejection attached so a
/// regression says which branch it came from.
fn directives(lookup: fn(&str) -> Result<String, VarError>) -> String {
    let resolved = resolve_log_level(lookup);
    assert_eq!(resolved.rejected, None, "expected the value to be accepted");
    resolved.directives
}

#[test]
fn log_level_defaults_to_info_when_unset() {
    let resolved = resolve_log_level(|_| Err(VarError::NotPresent));
    assert_eq!(resolved.directives, "info");
    // Genuinely unset, so there is nothing to report.
    assert_eq!(resolved.rejected, None);
}

#[test]
fn log_level_uses_the_configured_value() {
    assert_eq!(directives(|_| Ok("debug".to_string())), "debug");
}

#[test]
fn log_level_accepts_a_bare_off() {
    // The one bare level that turns everything off is still a deliberate choice.
    assert_eq!(directives(|_| Ok("off".to_string())), "off");
}

#[test]
fn log_level_keeps_multi_directive_values() {
    // The per-part check splits on `,`, so it could have rejected combinations
    // that used to work. Listed rather than looped: `resolve_log_level` takes a
    // `fn` pointer, so a loop variable cannot be captured.
    assert_eq!(
        directives(|_| Ok("info,cryptifier=debug".to_string())),
        "info,cryptifier=debug"
    );
    assert_eq!(directives(|_| Ok("debug,warn".to_string())), "debug,warn");
    assert_eq!(
        directives(|_| Ok("cryptifier=debug,off".to_string())),
        "cryptifier=debug,off"
    );
    assert_eq!(directives(|_| Ok("warn,[span]=trace".to_string())), "warn,[span]=trace");
    // Space after the comma, which the per-part trim has to tolerate.
    assert_eq!(directives(|_| Ok("info, debug".to_string())), "info, debug");
}

#[test]
fn log_level_rejects_a_typo_inside_a_multi_directive_value() {
    // The per-part check has to apply to every part, or a typo hidden behind a
    // valid first directive installs a filter matching nothing.
    let resolved = resolve_log_level(|_| Ok("info,deubg".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected.as_deref(), Some("info,deubg"));
}

#[test]
fn log_level_falls_back_when_the_configured_value_is_not_a_filter() {
    // `new` would install an ERROR-only filter for this, so reporting it
    // verbatim would be a lie about what is in effect.
    let resolved = resolve_log_level(|_| Ok("=debug".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected.as_deref(), Some("=debug"));
}

#[test]
fn log_level_treats_an_empty_value_as_unset() {
    // `EnvFilter::new("")` parses to no directives and installs an ERROR-only
    // filter — every startup line, including the one reporting this value,
    // would be suppressed.
    let resolved = resolve_log_level(|_| Ok(String::new()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected, None);
}

#[test]
fn log_level_treats_a_whitespace_only_value_as_unset() {
    let resolved = resolve_log_level(|_| Ok("   ".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected, None);
}

#[test]
fn log_level_trims_before_validating_and_reporting() {
    // A padded value parses fine, so the old code installed and printed the
    // padding — `Log level:  debug `. Nothing else in this file covers it,
    // because every other case is already trimmed.
    assert_eq!(directives(|_| Ok("  debug  ".to_string())), "debug");
}

#[test]
fn log_level_reports_a_value_that_is_not_valid_utf8() {
    // The key IS set here, just unreadable as text, so this is a config the
    // operator got wrong rather than one left alone.
    let resolved = resolve_log_level(|_| Err(VarError::NotUnicode(OsString::from("debug"))));
    assert_eq!(resolved.directives, "info");
    assert!(
        resolved.rejected.is_some(),
        "a set-but-unreadable value must still be reported, not defaulted silently"
    );
}

#[test]
fn log_level_rejects_a_misspelled_level() {
    // The reason this resolver exists. `EnvFilter` reads a bare word as a module
    // path, so `deubg` is valid, installs a filter matching nothing, and the
    // process logs nothing at all — measured at 0 lines against 9 for `info`.
    let resolved = resolve_log_level(|_| Ok("deubg".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected.as_deref(), Some("deubg"));
}

#[test]
fn log_level_rejects_a_real_target_written_as_a_bare_word() {
    // Deliberate narrowing: a genuine module name is indistinguishable from a
    // typo, and the same total silence applies. Use `cryptifier=trace` instead.
    let resolved = resolve_log_level(|_| Ok("cryptifier".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected.as_deref(), Some("cryptifier"));
}

#[test]
fn log_level_still_accepts_an_explicit_target_directive() {
    // The narrowing must not cost the documented flexible form.
    assert_eq!(directives(|_| Ok("tokio=trace".to_string())), "tokio=trace");
}

#[test]
fn log_level_accepts_a_level_in_any_case() {
    assert_eq!(directives(|_| Ok("INFO".to_string())), "INFO");
}

#[test]
fn log_level_rejects_a_trailing_comma() {
    // `LevelFilter::from_str("")` succeeds as `off`, so without an explicit
    // guard a trailing comma would clear the other branch's validation.
    let resolved = resolve_log_level(|_| Ok("info,".to_string()));
    assert_eq!(resolved.directives, "info");
    assert_eq!(resolved.rejected.as_deref(), Some("info,"));
}
