/// Represents the direction of a price change.
#[derive(Debug, Clone, PartialEq)]
pub enum PriceChange {
    Up,
    Down,
    NoChange,
}

impl PriceChange {
    pub fn as_text(&self) -> &'static str {
        match self {
            PriceChange::Up => "up",
            PriceChange::Down => "down",
            PriceChange::NoChange => "no change",
        }
    }
}
