# otc_pool

# 🎯 OTC Pool Program

### ***Overview***

The OTC Pool Program is a Solana based smart contract built with Anchor, designed to facilitate **semi-private OTC trading**, **instant direct swaps**, **whitelisted partner management**, and **escrow-based trade offers**. It enables flexible, secure, and customizable peer-to-peer trading ecosystems without intermediaries. 

This Program was made in Solana Playground IDE

---

### ***Key Features***

- **Pool Initialization**  
  Admins can create a new trading pool with dynamic partner limits, trading fees, treasury destinations, and minimum swap sizes.

- **Authority Management**  
  Pool administrators can securely transfer authority to another wallet if needed.

- **Partner Whitelisting**  
  Only approved partners can participate in trades, ensuring security and compliance.

- **Token Pair Control**  
  Admins can manage the list of allowed token pairs for swaps and OTC offers.

- **Direct Swaps**  
  Whitelisted partners can instantly exchange tokens without needing to escrow funds.

- **Escrowed OTC Offers**  
  Partners can create time-limited, semi-private OTC offers, with tokens escrowed safely until the trade is accepted or canceled.

- **Pause and Resume Functionality**  
  Admins can pause trading activities during emergencies and resume operations once the issue is resolved.

---

### ***Program Workflow 💬***

1. **Initialize Pool**  
   Define max partners, fee rates, treasury accounts, and minimum swap sizes.

2. **Manage Partners**  
   Add or remove whitelisted addresses who are eligible to participate in trades.

3. **Manage Supported Pairs**  
   Define and control which token pairs are tradable in the pool.

4. **Trading Options**
   - **Direct Swap**: Instant, atomic token swaps between two authorized partners.
   - **OTC Offers**: One partner escrows tokens, creates a public or semi-private offer, and another partner can accept it to complete the trade.

5. **Offer Lifecycle**
   - **Create Offer**: Lock tokens into escrow.
   - **Accept Offer**: Swap tokens, apply trading fee, and release funds.
   - **Cancel Offer**: Refund escrowed tokens before expiration.
   - **Extend Offer**: Lengthen the expiration deadline if needed.

6. **Emergency Controls**
   - Pause the entire pool to freeze all trades if needed.
   - Resume trading once the system is deemed safe.

---

### ***Event Tracking ✨***

The program emits events for all major actions, allowing easy monitoring and indexing:
- Pool creation
- Authority transfers
- Partner and token pair management
- Swap completions
- Offer creations, executions, cancellations, and extensions
- Pool pause and resume activities

---

### ***Error Handling***

Robust error codes are built-in to ensure safe, clear operation:
- Unauthorized access attempts
- Partner or pair limits being exceeded
- Invalid or unsupported trades
- Expired offers or offers already fulfilled
- Minimum swap requirements not met
- Invalid treasury accounts or improper trade extensions

---

### ***Customization & Security Highlights***

- **Flexible Configurations**: Dynamic settings for partner limits, fee percentages, and swap minimums.
- **Safe Whitelisting**: Only pre-approved wallets can participate in trading.
- **Efficient Fee Collection**: Fees are automatically collected into a treasury account.
- **Reliable Emergency Switches**: Admins can freeze trading when needed to respond to incidents.
- **Secure Escrows**: Tokens are safely held during OTC offer lifecycles, minimizing risk.

---

### ***Ideal Use Cases***

- Private OTC trading desks
- DAO treasury swaps
- Launchpads facilitating early-stage token deals
- Decentralized partner-only trading venues

---

### ***Conclusion***

The OTC Pool Program brings powerful, customizable OTC trading functionality to Solana with security-first design and extensive configurability. Whether for internal fund management or multi-party dealmaking, it provides all the building blocks needed for professional, flexible, and secure trading environments. 🚀
