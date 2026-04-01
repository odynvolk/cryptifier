use cryptifier::common::PriceChange;
use cryptifier::get_price_change::get_price_change;

#[test]
fn test_price_change_up_above_threshold() {
    let new_price = 110.0; // 10% increase from 100
    let threshold = 5.0; // 5% threshold

    let _ = get_price_change("test_up1", new_price, threshold);

    // Second call triggers price change
    let result = get_price_change("test_up1", 120.0, threshold); // 9.09% increase
    assert_eq!(result, PriceChange::Up);
}

#[test]
fn test_price_change_down_above_threshold() {
    let new_price = 90.0; // 10% decrease from 100
    let threshold = 5.0; // 5% threshold

    let _ = get_price_change("test_down1", 100.0, threshold);

    let result = get_price_change("test_down1", new_price, threshold);
    assert_eq!(result, PriceChange::Down);
}

#[test]
fn test_price_change_up_below_threshold() {
    let new_price = 103.0; // 3% increase
    let threshold = 5.0; // 5% threshold

    let _ = get_price_change("test_up2", 100.0, threshold);

    let result = get_price_change("test_up2", new_price, threshold);
    assert_eq!(result, PriceChange::NoChange);
}

#[test]
fn test_price_change_down_below_threshold() {
    let new_price = 97.0; // 3% decrease
    let threshold = 5.0; // 5% threshold

    let _ = get_price_change("test_down2", 100.0, threshold);

    let result = get_price_change("test_down2", new_price, threshold);
    assert_eq!(result, PriceChange::NoChange);
}

#[test]
fn test_price_change_exact_threshold() {
    let new_price = 105.0; // Exactly 5% increase
    let threshold = 5.0; // 5% threshold

    let _ = get_price_change("test_exact", 100.0, threshold);

    let result = get_price_change("test_exact", new_price, threshold);
    // Using > comparison, so exact threshold should return NoChange
    assert_eq!(result, PriceChange::NoChange);
}

#[test]
fn test_first_price_update() {
    let first_price = 100.0;
    let threshold = 5.0;

    let result = get_price_change("test_first", first_price, threshold);
    assert_eq!(result, PriceChange::NoChange);
}

#[test]
fn test_price_change_zero_threshold() {
    // Test with zero threshold (any change should trigger)
    let new_price = 100.01; // Very small increase
    let threshold = 0.0; // Zero threshold

    // First call sets initial price
    let _ = get_price_change("test_zero", 100.0, threshold);

    // Second call triggers price change
    let result = get_price_change("test_zero", new_price, threshold);
    // Should trigger Up since 0.01% > 0%
    assert_eq!(result, PriceChange::Up);
}
