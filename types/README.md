# aukla-types

The units shared by every Aukla crate.

- `Price`: a whole number of ticks. A tick is the smallest price step of a market.
- `Size`: a whole number of lots. A lot is the smallest amount that can be traded in a market.

All values are whole numbers, so every validator computes the same result. This crate depends on no other crate.