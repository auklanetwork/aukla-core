# Aukla

Aukla is an open proof-of-stake chain for public spot and perpetual markets.
Every market clears once per block, twice a second, at one price. Blocks carry
orders and never trades, so any node recomputes every price and every fill
from the rule.

The chain has a fixed set of actions: transfers, orders, margin, staking,
price reports and bridge transfers. It has no virtual machine.

## Design

[ARCHITECTURE.md](./ARCHITECTURE.md) describes how the implementation is
organised: the crates, what each one owns, and how a block is executed.

## Contributing

Issues and pull requests are welcome. A change to a protocol rule starts with
an issue.

## Licensing

This repository is dual-licensed under both the [Apache 2.0](./LICENSE-APACHE)
and [MIT](./LICENSE-MIT) licenses. You may choose either license when
employing this code.
