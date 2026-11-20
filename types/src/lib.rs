//! The units shared by every Aukla crate.
//!
//! All values are whole numbers, so every validator computes the same result.

/// A price, as a whole number of ticks.
/// A tick is the smallest price step of a market.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Price(u64);

impl Price {
    /// Creates a price of the given number of ticks.
    pub fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the number of ticks.
    pub fn ticks(self) -> u64 {
        self.0
    }
}

/// A size, as a whole number of lots.
/// A lot is the smallest amount that can be traded in a market.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Size(u64);

impl Size {
    /// Creates a size of the given number of lots.
    pub fn new(lots: u64) -> Self {
        Self(lots)
    }

    /// Returns the number of lots.
    pub fn lots(self) -> u64 {
        self.0
    }
}

/// The side of an order: buying or selling.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Returns the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_new_ticks() {
        let price = Price::new(1);
        assert_eq!(price.ticks(), 1);
    }

    #[test]
    fn price_ordering_follows_the_number() {
        let price_a = Price::new(1);
        let price_b = Price::new(2);
        assert!(price_a < price_b);
    }

    #[test]
    fn price_equal_values_are_equal() {
        let price_a = Price::new(1);
        let price_b = Price::new(1);
        assert_eq!(price_a, price_b);
    }

    #[test]
    fn size_new_lots() {
        let size = Size::new(1);
        assert_eq!(size.lots(), 1);
    }

    #[test]
    fn size_ordering_follows_the_number() {
        let size_a = Size::new(1);
        let size_b = Size::new(2);
        assert!(size_a < size_b);
    }

    #[test]
    fn size_equal_values_are_equal() {
        let size_a = Size::new(1);
        let size_b = Size::new(1);
        assert_eq!(size_a, size_b);
    }

    #[test]
    fn side_opposite_buy_to_sell() {
        let side = Side::Buy;
        assert_eq!(side.opposite(), Side::Sell);
    }

    #[test]
    fn side_opposite_sell_to_buy() {
        let side = Side::Sell;
        assert_eq!(side.opposite(), Side::Buy);
    }
}
