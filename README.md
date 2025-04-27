# otc_pool

## Overview

The OTC Pool Program is a Solana-based smart contract (program) built using **Anchor**, designed to facilitate:

- **Semi-private OTC trading**
- **Instant direct swaps**
- **Partner whitelisting and management**
- **Escrow-based trading offers**
- **Pool governance and emergency controls**

It enables **secure, flexible, and customizable peer-to-peer trading** without intermediaries, ideal for professional OTC desks, DAOs, or partner-only venues.

This program was fully developed and tested in **Solana Playground IDE**.

**Devnet deployment:**  
(https://explorer.solana.com/address/9s97f1eHD71SCRWCFVucTdEUPwwHEcPxWV9fDqE67EME?cluster=devnet)

---

# Key Features

- **Pool Initialization**  
  Create a pool with customizable max partners, fee rates (basis points), treasury accounts, minimum swap amounts, and expiration settings.

- **Authority and Treasury Management**  
  Transfer pool control (authority) or update treasury destination securely.

- **Partner Whitelisting**  
  Only approved wallet addresses can participate in trades.

- **Supported Token Pairs**  
  Define which token mints are allowed for OTC deals.

- **Direct Atomic Swaps**  
  Partners can instantly trade token-for-token without using escrow.

- **Escrowed OTC Offers**  
  Partners can escrow tokens into offers that other partners can accept before expiration.

- **Offer Extensions**  
  Makers can extend the expiration of active offers once.

- **Pool Pause/Resume Controls**  
  Admins can pause the pool during emergencies and resume when safe.

- **Offer Expiration and Force Close**  
  Offers can expire automatically or be manually closed if expired.

- **Customizable Fee Handling**  
  Fees from swaps and offers are automatically routed to the treasury.

---
