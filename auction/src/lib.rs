//! The clearing rule: from the offers of one market, the clearing price and
//! the fills.
//!
//! This crate knows nothing about balances, accounts or other markets.

use aukla_types::{BlockHeight, OfferId, Price, Side, Size};

/// An offer to buy or sell up to `size` lots in one market: a buy at
/// `limit` or lower, a sell at `limit` or higher.
///
/// New orders, orders resting from earlier blocks and quote sheet levels
/// all reach the auction as offers and are treated the same.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Offer {
    /// Identifies the offer. The last tie-break between offers that are
    /// otherwise equal.
    pub id: OfferId,
    /// Whether the offer buys or sells.
    pub side: Side,
    /// The worst price the offer accepts: the highest for a buy, the lowest
    /// for a sell.
    pub limit: Price,
    /// The most the offer will trade, in lots.
    pub size: Size,
    /// The block in which the offer was admitted. At the same limit, offers
    /// from earlier blocks fill first.
    pub admitted: BlockHeight,
}

/// The outcome of the auction for one market in one block.
///
/// Every fill in the market happens at `price`, and `volume` lots change
/// hands in total. A `Clearing` exists only when something trades.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Clearing {
    /// The one price of every fill in this market in this block.
    pub price: Price,
    /// The total lots that trade. Always above zero.
    pub volume: Size,
}

/// Why the auction could not produce a result.
///
/// A market where nothing trades is not an error. It is a normal result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuctionError {
    /// A total of offer sizes did not fit in a `u64`.
    Overflow,
}
