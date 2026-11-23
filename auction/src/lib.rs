//! The clearing rule: from the offers of one market, the clearing price and
//! the fills.
//!
//! This crate knows nothing about balances, accounts or other markets.

use std::cmp::Ordering;

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

/// Returns the total lots of all buys whose limit is `price` or higher, or
/// `Err(AuctionError::Overflow)` if the total overflows.
fn demand_at(offers: &[Offer], price: Price) -> Result<Size, AuctionError> {
    let mut total = Size::new(0);
    for o in offers {
        if o.side == Side::Buy && o.limit >= price {
            total = total.checked_add(o.size).ok_or(AuctionError::Overflow)?;
        }
    }
    Ok(total)
}

/// Returns the total lots of all sells whose limit is `price` or lower, or
/// `Err(AuctionError::Overflow)` if the total overflows.
fn supply_at(offers: &[Offer], price: Price) -> Result<Size, AuctionError> {
    let mut total = Size::new(0);
    for o in offers {
        if o.side == Side::Sell && o.limit <= price {
            total = total.checked_add(o.size).ok_or(AuctionError::Overflow)?;
        }
    }
    Ok(total)
}

/// Returns the number of lots that can trade at `price`.
///
/// This is the total of the buys or the total of the sells at that price,
/// whichever is smaller. Returns `Err(AuctionError::Overflow)` if either
/// total overflows.
fn volume_at(offers: &[Offer], price: Price) -> Result<Size, AuctionError> {
    let demand = demand_at(offers, price)?;
    let supply = supply_at(offers, price)?;
    Ok(demand.min(supply))
}

/// Finds the price at which the most lots can trade, and that number of
/// lots.
///
/// When several prices give the same largest volume, the clearing price is
/// the middle of the lowest and the highest of them, rounded down to a
/// whole tick.
///
/// Returns `Ok(None)` if there is no price at which anything can trade.
/// Returns `Err(AuctionError::Overflow)` if a total of sizes overflows.
pub fn clearing_price(offers: &[Offer]) -> Result<Option<Clearing>, AuctionError> {
    // The largest volume so far, and the lowest and the highest price that
    // give it.
    let mut best: Option<(Size, Price, Price)> = None;
    for o in offers {
        let new_price = o.limit;
        let new_vol = volume_at(offers, new_price)?;
        if new_vol == Size::new(0) {
            continue;
        }
        match best {
            None => {
                best = Some((new_vol, new_price, new_price));
            }
            Some((volume, lowest, highest)) => match new_vol.cmp(&volume) {
                Ordering::Greater => {
                    best = Some((new_vol, new_price, new_price));
                }
                Ordering::Equal => {
                    best = Some((new_vol, lowest.min(new_price), highest.max(new_price)));
                }
                Ordering::Less => {}
            },
        }
    }
    match best {
        None => Ok(None),
        Some((volume, lowest, highest)) => Ok(Some(Clearing {
            price: lowest.midpoint(highest),
            volume,
        })),
    }
}

#[cfg(test)]
mod tests {
    use aukla_types::Digest;

    use super::*;

    fn buy(n: u8, limit: u64, size: u64) -> Offer {
        Offer {
            id: OfferId::new(Digest::new([n; 32])),
            side: Side::Buy,
            limit: Price::new(limit),
            size: Size::new(size),
            admitted: BlockHeight::new(1),
        }
    }

    fn sell(n: u8, limit: u64, size: u64) -> Offer {
        Offer {
            id: OfferId::new(Digest::new([n; 32])),
            side: Side::Sell,
            limit: Price::new(limit),
            size: Size::new(size),
            admitted: BlockHeight::new(1),
        }
    }

    #[test]
    fn demand_at_counts_buys_at_or_above_price() {
        let a = buy(1, 2000, 10);
        let b = buy(2, 1999, 5);
        let c = buy(3, 1998, 7);
        let d = buy(4, 2001, 4);
        let e = sell(5, 1999, 4);
        let book = [a, b, c, d, e];
        assert_eq!(demand_at(&book, Price::new(1998)), Ok(Size::new(26)));
        assert_eq!(demand_at(&book, Price::new(1999)), Ok(Size::new(19)));
        assert_eq!(demand_at(&book, Price::new(2000)), Ok(Size::new(14)));
        assert_eq!(demand_at(&book, Price::new(2001)), Ok(Size::new(4)));
    }

    #[test]
    fn demand_at_price_above_all_buys_gives_zero() {
        let a = buy(1, 2000, 10);
        let b = buy(2, 1999, 5);
        let c = buy(3, 1998, 7);
        assert_eq!(demand_at(&[a, b, c], Price::new(2001)), Ok(Size::new(0)));
    }

    #[test]
    fn demand_at_ignores_sells() {
        let a = sell(1, 2000, 10);
        let b = sell(2, 1999, 5);
        let c = sell(3, 1998, 7);
        assert_eq!(demand_at(&[a, b, c], Price::new(1998)), Ok(Size::new(0)));
    }

    #[test]
    fn demand_at_overflow_gives_error() {
        let a = buy(1, 2000, u64::MAX);
        let b = buy(2, 1999, 1);
        assert_eq!(
            demand_at(&[a, b], Price::new(1999)),
            Err(AuctionError::Overflow)
        );
    }

    #[test]
    fn supply_at_counts_sells_at_or_below_price() {
        let a = sell(1, 2000, 10);
        let b = sell(2, 1999, 5);
        let c = sell(3, 1998, 7);
        let d = sell(4, 2001, 4);
        let e = buy(5, 1999, 4);
        let book = [a, b, c, d, e];
        assert_eq!(supply_at(&book, Price::new(2001)), Ok(Size::new(26)));
        assert_eq!(supply_at(&book, Price::new(2000)), Ok(Size::new(22)));
        assert_eq!(supply_at(&book, Price::new(1999)), Ok(Size::new(12)));
        assert_eq!(supply_at(&book, Price::new(1998)), Ok(Size::new(7)));
    }

    #[test]
    fn supply_at_price_below_all_sells_gives_zero() {
        let a = sell(1, 2000, 10);
        let b = sell(2, 1999, 5);
        let c = sell(3, 1998, 7);
        assert_eq!(supply_at(&[a, b, c], Price::new(1997)), Ok(Size::new(0)));
    }

    #[test]
    fn supply_at_ignores_buys() {
        let a = buy(1, 2000, 10);
        let b = buy(2, 1999, 5);
        let c = buy(3, 1998, 7);
        assert_eq!(supply_at(&[a, b, c], Price::new(2000)), Ok(Size::new(0)));
    }

    #[test]
    fn supply_at_overflow_gives_error() {
        let a = sell(1, 2000, u64::MAX);
        let b = sell(2, 1999, 1);
        assert_eq!(
            supply_at(&[a, b], Price::new(2000)),
            Err(AuctionError::Overflow)
        );
    }

    #[test]
    fn volume_at_is_lower_of_demand_and_supply() {
        let a = buy(1, 2000, 10);
        let b = sell(2, 1999, 5);
        let c = sell(3, 1998, 7);
        let d = buy(4, 2001, 4);
        let e = buy(5, 1999, 4);
        let book = [a, b, c, d, e];
        assert_eq!(volume_at(&book, Price::new(1998)), Ok(Size::new(7)));
        assert_eq!(volume_at(&book, Price::new(1999)), Ok(Size::new(12)));
        assert_eq!(volume_at(&book, Price::new(2000)), Ok(Size::new(12)));
        assert_eq!(volume_at(&book, Price::new(2001)), Ok(Size::new(4)));
    }

    #[test]
    fn volume_at_demand_overflow_gives_error() {
        let a = buy(1, 2000, u64::MAX);
        let b = buy(2, 1999, 1);
        assert_eq!(
            volume_at(&[a, b], Price::new(1999)),
            Err(AuctionError::Overflow)
        );
    }

    #[test]
    fn volume_at_supply_overflow_gives_error() {
        let a = sell(1, 2000, u64::MAX);
        let b = sell(2, 1999, 1);
        assert_eq!(
            volume_at(&[a, b], Price::new(2000)),
            Err(AuctionError::Overflow)
        );
    }

    #[test]
    fn clearing_price_finds_price_with_most_volume() {
        let a = buy(1, 102, 5);
        let b = sell(2, 101, 6);
        let c = buy(3, 101, 3);
        let d = sell(4, 103, 5);
        let e = buy(5, 100, 4);
        let f = sell(6, 99, 2);
        let offers = [a, b, c, d, e, f];
        assert_eq!(
            clearing_price(&offers),
            Ok(Some(Clearing {
                price: Price::new(101),
                volume: Size::new(8)
            }))
        );
    }

    #[test]
    fn clearing_price_tie_gives_middle() {
        let a = buy(1, 102, 6);
        let b = sell(2, 100, 6);
        assert_eq!(
            clearing_price(&[a, b]),
            Ok(Some(Clearing {
                price: Price::new(101),
                volume: Size::new(6)
            }))
        );
    }

    #[test]
    fn clearing_price_tie_rounds_middle_down() {
        let a = buy(1, 103, 6);
        let b = sell(2, 100, 6);
        assert_eq!(
            clearing_price(&[a, b]),
            Ok(Some(Clearing {
                price: Price::new(101),
                volume: Size::new(6)
            }))
        );
    }

    #[test]
    fn clearing_price_empty_book_gives_none() {
        assert_eq!(clearing_price(&[]), Ok(None));
    }

    #[test]
    fn clearing_price_no_overlap_gives_none() {
        let a = buy(1, 100, 2);
        let b = sell(2, 102, 4);
        assert_eq!(clearing_price(&[a, b]), Ok(None));
    }

    #[test]
    fn clearing_price_one_side_only_gives_none() {
        let a = buy(1, 100, 2);
        let b = buy(2, 102, 4);
        assert_eq!(clearing_price(&[a, b]), Ok(None));
    }

    #[test]
    fn clearing_price_overflow_gives_error() {
        let a = buy(1, 102, u64::MAX);
        let b = buy(2, 101, 1);
        let c = sell(3, 100, 3);
        assert_eq!(clearing_price(&[a, b, c]), Err(AuctionError::Overflow));
    }

    #[test]
    fn clearing_price_paper_example() {
        let a = buy(1, 2004, 4);
        let b = buy(2, 2002, 6);
        let c = buy(3, 1999, 5);
        let d = sell(4, 1998, 3);
        let e = sell(5, 2000, 4);
        let f = sell(6, 2000, 4);
        let g = sell(7, 2000, 2);
        let h = sell(8, 2003, 5);
        let offers = [a, b, c, d, e, f, g, h];
        assert_eq!(
            clearing_price(&offers),
            Ok(Some(Clearing {
                price: Price::new(2001),
                volume: Size::new(10)
            }))
        );
    }
}
