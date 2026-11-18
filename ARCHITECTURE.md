# Architecture

This document describes how the Aukla implementation is organised: the crates,
what each one owns, how they depend on each other, and how a block is
executed.

## The core and the shell

The chain is a function:

    apply(state, block) -> state

A block is a list of signed actions: orders, cancels, transfers, price reports
and so on. Applying it gives the next state, including every clearing price
and every fill. The function reads nothing but its two arguments. It has no
network, no clock, no disk and no randomness.

The code is split along that line.

- The **core** is everything inside `apply`. It is deterministic and
  synchronous. It decides what is valid and what happens.
- The **shell** is everything around it: consensus, networking, the mempool,
  storage and the API. It decides nothing about prices, fills or balances. It
  delivers blocks to the core and stores what comes back.

Two properties follow. Any machine that applies the same blocks reaches the
same state, which is what lets anyone check the chain. And each part of the
core takes values and returns values, so it is tested without a network or a
database.

## Layout

Each crate is a directory at the top of the repository, with its own README.
The validator program is the `validator` crate, a thin binary around `node`.

## Crates

| Crate | Owns | Leaves to others |
|---|---|---|
| `types` | Identifiers, amounts, prices in ticks, sizes in lots, the action formats and their wire encoding | Every rule |
| `auction` | The clearing rule: from the orders of one market, the clearing price and the fills | Balances, accounts, other markets |
| `ledger` | Who holds what: balances, reserves, deposits, transfers, fee collection | Why a balance moves |
| `market` | The state of one market between blocks: resting orders, quote sheets, expiry. Runs `auction` and returns the fills | Settling the fills |
| `oracle` | The outside price: validator price reports, freshness, the stake-weighted median | Using the price |
| `perps` | Positions, margin, funding, liquidation orders, insurance funds and the cover rule | Clearing. A liquidation is an order like any other |
| `admission` | What a block may contain: the fixed charges, the work cost of each action, the work budget, the sharing rule for a full block, inclusion-list checks | Choosing which pending actions to propose, which is the mempool's job |
| `staking` | The validator set, stake pools, rewards and penalties | Consensus itself |
| `bridge` | The Ethereum gateway as the chain sees it: verifying deposit proofs, replay protection, withdrawal limits | Talking to Ethereum |
| `execution` | `apply(state, block)`. Authorises each action, runs the phases of a block in their fixed order, routes fills to `ledger` and `perps`, and computes the state root | The rules that belong to a single crate above |
| `node` | The shell: consensus, peer-to-peer, the mempool, storage, the API, and the relayers for price reports and the gateway | Every decision about prices, fills and balances |

## Dependencies

Each crate lists everything it may import from the workspace.

| Crate | Depends on |
|---|---|
| `types` | nothing |
| `auction` | `types` |
| `ledger` | `types` |
| `oracle` | `types` |
| `market` | `types`, `auction` |
| `perps` | `types`, `ledger` |
| `admission` | `types`, `ledger` |
| `staking` | `types`, `ledger` |
| `bridge` | `types`, `ledger` |
| `execution` | every crate above |
| `node` | `types`, `execution` |

When one core crate needs a value that another one owns, `execution` reads it
from the owner and passes it in. `perps` receives the outside price as an
argument and does not import `oracle`. `bridge` receives the slashable stake
as an argument and does not import `staking`. The list above therefore has no
cycles and stays short.

## One block

`execution` executes a block in fixed phases. The order is part of the protocol:
two nodes that ran the phases in a different order would disagree.

1. **Admit.** Check each action's signature and authority, take its charge,
   and check the block against the work budget, the sharing rule and the
   inclusion lists. A block that fails any of these is invalid as a whole.
2. **Apply actions.** Transfers, deposits and withdrawals, staking actions,
   price reports, cancels, quote sheet replacements and new orders. A new
   order reserves what it could spend.
3. **Outside price.** Aggregate the block's price reports per asset.
4. **Positions.** Accrue funding, check margin at the outside price, and place
   a liquidation order for every position below maintenance margin.
5. **Clear.** Run the auction for every market. Markets share nothing inside
   this phase, so they clear in parallel and the result does not depend on the
   order in which they finish.
6. **Settle.** Route every fill: spot fills move balances in `ledger`,
   perpetual fills change positions in `perps`. Collect trading fees and
   distribute them.
7. **Close.** Expire orders, apply end-of-epoch staking changes, and compute
   the state root.

## The node

The node is built on the Commonware library. It orders blocks with Simplex
consensus and hands each one to `execution`.

- **Consensus** agrees on the order of blocks. A proposed block is checked by
  running `apply` on it, so a validator votes only for a block the core
  accepts.
- **The mempool** holds pending actions and chooses which to propose. It uses
  `admission` through `execution` to build a block that is valid, and it cannot
  make an invalid block valid.
- **Storage** keeps the state, recent blocks and the consensus records needed
  to verify them. It keeps the state of blocks that are proposed but not yet
  final separate from final state, and it recovers to the last final block
  after a crash.
- **The API** serves the state and the market data that `apply` produces:
  every order, every fill and the book after each block.
- **Relayers** fetch exchange prices and Ethereum headers and submit them as
  ordinary signed actions. The core treats them like any other input.

## Rules for the core

Every crate from `types` to `execution` follows these.

- **Deterministic.** No floating point, no system time, no randomness, and no
  iteration over a collection whose order is not defined.
- **Integers only.** Prices are in ticks and sizes in lots. Every rounding is
  written out where it happens.
- **Synchronous and free of I/O.** Functions take state and return state.
- **Errors, never panics, on input.** Anything a user can send returns an
  error. A panic is a bug in the chain.
- **No unsafe code.**
- **Checked by worked examples.** Each rule has a worked example with real
  numbers, kept as a test in the crate that implements it.

## Outside this repository

The gateway contract on Ethereum is written in Solidity and has its own
repository.
