//! The basic types shared by every Aukla crate.
//!
//! All values are whole numbers, so every validator computes the same result.

/// A price, as a whole number of ticks.
///
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
///
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

    /// Adds two sizes and returns the sum, or `None` if the sum overflows.
    pub fn checked_add(self, other: Size) -> Option<Size> {
        Some(Self(self.0.checked_add(other.0)?))
    }
}

/// The side of an order: buying or selling.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Side {
    /// An order to buy.
    Buy,
    /// An order to sell.
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

/// The number of a block, counted from the first block.
///
/// Gives an order its time priority.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BlockHeight(u64);

impl BlockHeight {
    /// Creates a block height from the given block number.
    pub fn new(height: u64) -> Self {
        Self(height)
    }

    /// Returns the block number.
    pub fn get(self) -> u64 {
        self.0
    }
}

/// 32 bytes produced by a hash.
///
/// Compares byte by byte from the first byte.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Digest([u8; 32]);

impl Digest {
    /// Creates a digest from the given 32 bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrows the 32 bytes without copying them.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The identifier of one offer in an auction.
///
/// It is the last tie-break when two offers are otherwise equal.
/// Wraps a [`Digest`] and orders the same way.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OfferId(Digest);

impl OfferId {
    /// Creates an offer id from the given digest.
    pub fn new(digest: Digest) -> Self {
        Self(digest)
    }

    /// Returns the digest.
    pub fn get(self) -> Digest {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A digest that is all zeros except for its first and last byte.
    fn digest(first: u8, last: u8) -> Digest {
        let mut bytes = [0u8; 32];
        bytes[0] = first;
        bytes[31] = last;
        Digest::new(bytes)
    }

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
    fn size_checked_add_gives_sum() {
        let size_a = Size::new(3);
        let size_b = Size::new(4);
        assert_eq!(size_a.checked_add(size_b), Some(Size::new(7)));
    }

    #[test]
    fn size_checked_add_overflow_gives_none() {
        let size_a = Size::new(u64::MAX);
        let size_b = Size::new(1);
        assert_eq!(size_a.checked_add(size_b), None);
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

    #[test]
    fn block_height_new_get() {
        let height = BlockHeight::new(7);
        assert_eq!(height.get(), 7);
    }

    #[test]
    fn block_height_ordering_follows_the_number() {
        let height_a = BlockHeight::new(1);
        let height_b = BlockHeight::new(2);
        assert!(height_a < height_b);
    }

    #[test]
    fn block_height_equal_values_are_equal() {
        let height_a = BlockHeight::new(1);
        let height_b = BlockHeight::new(1);
        assert_eq!(height_a, height_b);
    }

    #[test]
    fn digest_new_as_bytes() {
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        bytes[15] = 2;
        bytes[31] = 3;
        let digest = Digest::new(bytes);
        assert_eq!(digest.as_bytes(), &bytes);
    }

    #[test]
    fn digest_ordering_sees_the_first_byte() {
        assert!(digest(0, 0) < digest(1, 0));
    }

    #[test]
    fn digest_ordering_sees_the_last_byte() {
        assert!(digest(0, 0) < digest(0, 1));
    }

    #[test]
    fn digest_ordering_first_byte_decides() {
        assert!(digest(0, 9) < digest(1, 0));
    }

    #[test]
    fn digest_equal_bytes_are_equal() {
        assert_eq!(digest(1, 2), digest(1, 2));
    }

    #[test]
    fn digest_different_bytes_are_not_equal() {
        assert_ne!(digest(1, 2), digest(1, 3));
    }

    #[test]
    fn offer_id_new_get() {
        let offer_id = OfferId::new(digest(1, 2));
        assert_eq!(offer_id.get(), digest(1, 2));
    }

    #[test]
    fn offer_id_ordering_follows_the_digest() {
        let offer_id_a = OfferId::new(digest(0, 9));
        let offer_id_b = OfferId::new(digest(1, 0));
        assert!(offer_id_a < offer_id_b);
    }

    #[test]
    fn offer_id_equal_digests_are_equal() {
        let offer_id_a = OfferId::new(digest(1, 2));
        let offer_id_b = OfferId::new(digest(1, 2));
        assert_eq!(offer_id_a, offer_id_b);
    }
}
