# aukla-types

The basic types shared by every Aukla crate.

- `Price`: a whole number of ticks. A tick is the smallest price step of a market.
- `Size`: a whole number of lots. A lot is the smallest amount that can be traded in a market.
- `Side`: buy or sell.
- `BlockHeight`: the number of a block, counted from the first block. Gives an order its time priority.
- `Digest`: 32 bytes produced by a hash. Compares byte by byte from the first byte.
- `OfferId`: the identifier of one offer in an auction, and the last tie-break when two offers are otherwise equal.

All values are whole numbers, so every validator computes the same result. This crate depends on no other crate.
