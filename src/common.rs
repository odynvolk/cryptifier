/// Represents the direction of a price change.
#[derive(Debug, Clone, PartialEq)]
pub enum PriceChange {
    Up,
    Down,
    NoChange,
}
