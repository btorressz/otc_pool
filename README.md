# otc_pool

# OTC Pool Program

This repository contains an **OTC Pool Program** written in Rust using the [Anchor Framework](https://book.anchor-lang.com/). The program implements a decentralized Over-the-Counter (OTC) token swapping mechanism on the Solana blockchain. Users can create and manage pools, whitelist partners, and perform secure token swaps. This program was made in Solana Playground IDE 

## Features

- **Pool Management**:
  - Initialize a pool with customizable parameters: max partners, fee basis points, treasury address, and minimum swap amount.
  - Transfer pool authority to a new owner.
  - Pause and resume pool operations in case of emergencies.

- **Partner Management**:
  - Whitelist partners (up to a configurable limit).
  - Remove partners from the whitelist.

- **Token Pair Management**:
  - Add supported token pairs for swaps.
  - Remove token pairs from the supported list.

- **Swapping**:
  - Perform direct swaps between whitelisted partners with enforced minimum swap amounts.
  - Create OTC offers with escrow, allowing semi-private swaps with expiration and fees.
  - Cancel or extend OTC offers before expiration.

- **Fee and Treasury**:
  - Collect swap fees into a designated treasury account.
