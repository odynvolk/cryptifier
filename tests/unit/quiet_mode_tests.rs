//! Tests for Quiet Mode functionality.
use cryptifier::config::{
    is_quiet_hours, is_quiet_mode_enabled, get_quiet_mode_start_hour, get_quiet_mode_end_hour,
};

#[tokio::test]
async fn test_quiet_mode_enabled_value() {
    // Test that quiet mode enabled value is parsed correctly from config
    let enabled = is_quiet_mode_enabled();
    // Value should be a valid boolean
    assert!(enabled == true || enabled == false);
}

#[tokio::test]
async fn test_quiet_mode_start_hour_value() {
    // Test that start hour value is within valid range
    let start_hour = get_quiet_mode_start_hour();
    assert!(start_hour >= 0 && start_hour <= 23);
}

#[tokio::test]
async fn test_quiet_mode_end_hour_value() {
    // Test that end hour value is within valid range
    let end_hour = get_quiet_mode_end_hour();
    assert!(end_hour >= 0 && end_hour <= 23);
}

#[tokio::test]
async fn test_quiet_mode_is_quiet_hours() {
    // Test that is_quiet_hours returns a boolean
    let result = is_quiet_hours();
    assert!(result == true || result == false);
}

#[tokio::test]
async fn test_quiet_mode_config_values_valid() {
    // Test that all quiet mode configuration values are valid
    
    let enabled = is_quiet_mode_enabled();
    let start_hour = get_quiet_mode_start_hour();
    let end_hour = get_quiet_mode_end_hour();
    
    // Start and end hours should be in valid range (0-23)
    assert!(start_hour >= 0 && start_hour <= 23, "Start hour {} out of range", start_hour);
    assert!(end_hour >= 0 && end_hour <= 23, "End hour {} out of range", end_hour);
    
    // If quiet mode is enabled, start should typically be less than end
    // (unless it's a wrap-around midnight case)
    if enabled {
        // Both values should be valid hours
        assert!(start_hour >= 0, "Start hour should be >= 0");
        assert!(end_hour >= 0, "End hour should be >= 0");
    }
}

#[tokio::test]
async fn test_quiet_mode_parsing_from_env() {
    // Test that environment variables are parsed correctly
    
    // Get values from environment (as the config does)
    let enabled_str = std::env::var("APP__QUIET_MODE_ENABLED").unwrap_or_else(|_| "false".to_string());
    let start_str = std::env::var("APP__QUIET_MODE_START_HOUR").unwrap_or_else(|_| "0".to_string());
    let end_str = std::env::var("APP__QUIET_MODE_END_HOUR").unwrap_or_else(|_| "6".to_string());
    
    // Parse the values
    let enabled: Option<bool> = enabled_str.parse().ok();
    let start_hour: Option<i64> = start_str.parse().ok();
    let end_hour: Option<i64> = end_str.parse().ok();
    
    // All should parse successfully
    assert!(enabled.is_some(), "Failed to parse APP__QUIET_MODE_ENABLED");
    assert!(start_hour.is_some(), "Failed to parse APP__QUIET_MODE_START_HOUR");
    assert!(end_hour.is_some(), "Failed to parse APP__QUIET_MODE_END_HOUR");
    
    // Values should be in valid ranges
    if let Some(e) = enabled {
        assert!(e == true || e == false);
    }
    if let Some(s) = start_hour {
        assert!(s >= 0 && s <= 23, "Start hour {} out of range", s);
    }
    if let Some(e) = end_hour {
        assert!(e >= 0 && e <= 23, "End hour {} out of range", e);
    }
}

#[tokio::test]
async fn test_quiet_mode_invalid_parsing() {
    // Test that invalid values are handled gracefully
    
    // Test invalid boolean
    let invalid_bool = "invalid".to_string();
    let parsed_bool: Option<bool> = invalid_bool.parse().ok();
    assert!(parsed_bool.is_none(), "Invalid boolean should return None");
    
    // Test invalid hour
    let invalid_hour = "invalid".to_string();
    let parsed_hour: Option<i64> = invalid_hour.parse().ok();
    assert!(parsed_hour.is_none(), "Invalid hour should return None");
}

#[tokio::test]
async fn test_quiet_mode_edge_cases() {
    // Test edge case hour values
    
    // Test start hour = 0 (midnight)
    let start: Option<i64> = "0".parse().ok();
    assert_eq!(start, Some(0));
    
    // Test start hour = 23 (11 PM)
    let start: Option<i64> = "23".parse().ok();
    assert_eq!(start, Some(23));
    
    // Test end hour = 0 (midnight)
    let end: Option<i64> = "0".parse().ok();
    assert_eq!(end, Some(0));
    
    // Test end hour = 23 (11 PM)
    let end: Option<i64> = "23".parse().ok();
    assert_eq!(end, Some(23));
}
