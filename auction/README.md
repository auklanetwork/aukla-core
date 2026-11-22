# aukla-auction

The clearing rule: from the offers of one market, the clearing price and the fills.

- `Offer`: an offer to buy or sell up to a size at a limit price or better. New orders, resting orders and quote sheet levels all reach the auction as offers.
- `Clearing`: the outcome for one market in one block. One price for every fill, and the total lots that trade.
- `AuctionError`: why the auction could not produce a result. A market where nothing trades is not an error.

This crate knows nothing about balances, accounts or other markets. It depends only on `aukla-types`.
