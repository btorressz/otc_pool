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

## Program Workflow 💬

1. **Initialize Pool**  
   Set admin parameters like max partners, treasury account, min swap size, expiration rules.

2. **Manage Whitelist and Token Pairs**  
   Add/remove approved partners.  
   Add/remove supported token pairs.

3. **Trading Options**
   - **Direct Swap:** Instant swap between two whitelisted partners.
   - **OTC Offer:**  
     - Maker escrows tokens and sets offer terms.
     - Taker accepts the offer before expiration to execute trade.

4. **Offer Lifecycle**
   - **Create Offer:** Escrow tokens and define terms.
   - **Accept Offer:** Swap tokens and collect treasury fee.
   - **Cancel Offer:** Refund escrowed tokens before expiration.
   - **Extend Offer:** Push out expiration once if needed.

5. **Emergency Controls**
   - Pause pool to freeze all trades.
   - Resume trading when incident is resolved.

6. **Expiration Handling**
   - Expired offers can be closed manually by makers or automatically force-closed later (future upgrade-ready).

---

## Event Tracking ✨

The program emits events for all critical actions:

- `PoolInitialized`
- `AuthorityTransferred`
- `TreasuryUpdated`
- `MintWhitelisted`
- `MintRemoved`
- `SupportedPairAdded`
- `SupportedPairRemoved`
- `PartnerAdded`
- `PartnerRemoved`
- `SwapDirectExecuted`
- `OfferCreated`
- `OfferCancelled`
- `OfferExecuted`
- `OfferExtended`
- `OfferExpired`
- `PoolPaused`
- `PoolResumed`

This makes the OTC Pool **indexer-friendly** and easy to integrate into frontends, dashboards, and trading history UIs.

---

## Error Handling 🚫

Robust error codes ensure safe and predictable behavior:

| Error Code                  | Description                                  |
|:-----------------------------|:---------------------------------------------|
| `Unauthorized`               | Caller lacks permission |
| `PartnerLimitReached`        | Max partners already added |
| `PartnerAlreadyExists`       | Partner already whitelisted |
| `PartnerNotFound`            | Partner not in whitelist |
| `PairAlreadyExists`          | Token pair already supported |
| `PairNotFound`               | Supported token pair not found |
| `PoolIsPaused`               | Pool is paused and cannot trade |
| `UnauthorizedPartner`        | Caller is not a whitelisted partner |
| `OfferAlreadyFulfilled`      | Offer already executed |
| `OfferExpired`               | Offer expired and can't be accepted |
| `OfferNotExpired`            | Offer still active (for expiration ops) |
| `SwapBelowMinimum`           | Swap amount too small |
| `InvalidFillAmount`          | Attempted to overfill or underfill offer |
| `MintAlreadyWhitelisted`     | Token mint already in whitelist |
| `MintNotWhitelisted`         | Token mint not found in whitelist |
| `InvalidTreasuryAccount`     | Bad treasury account provided |
| `InvalidExtension`           | Invalid attempt to extend offer |
| `ExpirationTooLong`          | Offer expiration exceeds allowed max |

---

## Security & Governance Highlights 🛡️

- **Flexible Configuration:**  
  Pool admins can update critical parameters (fees, treasury, partner limits) over time.

- **Safe Whitelisting:**  
  Only vetted partners can access OTC operations.

- **Escrow Safety:**  
  Tokens are escrowed securely using PDAs (Program Derived Addresses).

- **Emergency Controls:**  
  Trading can be paused instantly if suspicious activity or vulnerabilities are detected.

- **Extensibility:**  
  The architecture supports easy additions like SOL-native swaps, offer metadata, partner fee tiers, pre-signed offers, and more.

---

## Ideal Use Cases

- OTC desks operating private token deals
- DAO treasuries swapping tokens between trusted parties
- Launchpads running semi-private seed/strategic swaps
- Protocol partners managing liquidity swaps securely
- Institutional partners or market makers trading off-market

---

## Conclusion 🚀

The **OTC Pool Program** empowers **secure, customizable, semi-private OTC trading** on Solana.  
It combines **flexibility**, **security-first design**, and **powerful governance tools** to enable institutional-grade OTC dealmaking for the modern DeFi ecosystem.

Whether you're managing a DAO treasury, a launchpad, or a private trading desk —  
this program gives you the **on-chain tools needed to operate safely and efficiently**.

---

