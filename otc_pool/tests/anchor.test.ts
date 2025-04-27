// No imports needed: web3, anchor, pg and more are globally available

describe("OTC Pool Tests", () => {
  it("initialize_pool", async () => {
    // Generate a keypair for the pool
    const poolKeypair = new web3.Keypair();

    // Set parameters for the pool initialization
    const maxPartners = 5; // Maximum number of partners (u8)
    const feeBps = 100; // Fee in basis points (u16)
    const treasury = new web3.Keypair().publicKey; // Treasury public key (PublicKey)
    const minSwapAmount = new BN(1000); // Minimum swap amount (u64)
    const maxExpirationSecs = new BN(3600); // Maximum expiration time in seconds (i64)
    const partners = []; // Empty array for initial partners (PublicKey[])
    const initialWhitelistedMints = [new web3.Keypair().publicKey]; // Example whitelisted mints (PublicKey[])

    // Send the transaction to initialize the pool
    const txHash = await pg.program.methods
      .initializePool(
        maxPartners,
        feeBps,
        treasury,
        minSwapAmount,
        maxExpirationSecs,
        initialWhitelistedMints // Pass the new parameter here
      )
      .accounts({
        pool: poolKeypair.publicKey,
        authority: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([poolKeypair])
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm the transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the initialized pool account
    const poolAccount = await pg.program.account.pool.fetch(poolKeypair.publicKey);

    console.log("Pool initialized with data:", {
      authority: poolAccount.authority.toBase58(),
      maxPartners: poolAccount.maxPartners,
      feeBps: poolAccount.feeBps,
      treasury: poolAccount.treasury.toBase58(),
      minSwapAmount: poolAccount.minSwapAmount.toString(),
      maxExpirationSecs: poolAccount.maxExpirationSecs.toString(),
      initialWhitelistedMints: poolAccount.whitelistedMints.map((mint: web3.PublicKey) => mint.toBase58()),
      partners: poolAccount.partners.map((p: web3.PublicKey) => p.toBase58()),
    });

    // Validate the parameters
    assert.strictEqual(poolAccount.authority.toBase58(), pg.wallet.publicKey.toBase58());
    assert.strictEqual(poolAccount.maxPartners, maxPartners);
    assert.strictEqual(poolAccount.feeBps, feeBps);
    assert.strictEqual(poolAccount.treasury.toBase58(), treasury.toBase58());
    assert(minSwapAmount.eq(new BN(poolAccount.minSwapAmount)));
    assert(maxExpirationSecs.eq(new BN(poolAccount.maxExpirationSecs)));
    assert.deepStrictEqual(poolAccount.whitelistedMints.map((mint: web3.PublicKey) => mint.toBase58()), initialWhitelistedMints.map((mint: web3.PublicKey) => mint.toBase58()));
    assert.deepStrictEqual(poolAccount.partners, partners);
  });
});
