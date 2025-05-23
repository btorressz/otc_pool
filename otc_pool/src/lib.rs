use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_program::{program::invoke, system_instruction};


declare_id!("9s97f1eHD71SCRWCFVucTdEUPwwHEcPxWV9fDqE67EME");

pub const NATIVE_MINT: Pubkey = Pubkey::new_from_array([0u8; 32]);

#[program]
pub mod otc_pool {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        max_partners: u8,
        fee_bps: u16,
        treasury: Pubkey,
        min_swap_amount: u64,
        max_expiration_secs: i64,
        initial_whitelisted_mints: Vec<Pubkey>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = *ctx.accounts.authority.key;
        pool.max_partners = max_partners;
        pool.partners = Vec::new();
        pool.whitelisted_mints = initial_whitelisted_mints;
        pool.supported_pairs = Vec::new();
        pool.paused = false;
        pool.fee_bps = fee_bps;
        pool.treasury = treasury;
        pool.min_swap_amount = min_swap_amount;
        pool.max_expiration_secs = max_expiration_secs;
        emit!(PoolInitialized {
            authority: pool.authority,
            max_partners,
            fee_bps,
            treasury,
            min_swap_amount,
            max_expiration_secs,
        });
        Ok(())
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        let previous = pool.authority;
        pool.authority = new_authority;
        emit!(AuthorityTransferred { previous, new: new_authority });
        Ok(())
    }

    pub fn update_treasury(ctx: Context<TransferAuthority>, new_treasury: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        let previous = pool.treasury;
        pool.treasury = new_treasury;
        emit!(TreasuryUpdated { previous, new: new_treasury });
        Ok(())
    }

    pub fn add_whitelisted_mint(ctx: Context<ModifyPoolMints>, mint: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        require!(!pool.whitelisted_mints.contains(&mint), OtcError::MintAlreadyWhitelisted);
        pool.whitelisted_mints.push(mint);
        emit!(MintWhitelisted { mint });
        Ok(())
    }

    pub fn remove_whitelisted_mint(ctx: Context<ModifyPoolMints>, mint: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        if let Some(i) = pool.whitelisted_mints.iter().position(|x| *x == mint) {
            pool.whitelisted_mints.swap_remove(i);
            emit!(MintRemoved { mint });
            Ok(())
        } else {
            err!(OtcError::MintNotWhitelisted)
        }
    }

    pub fn close_expired_offer(ctx: Context<CloseExpiredOffer>) -> Result<()> {
        let offer = &mut ctx.accounts.offer;
        require!(!offer.fulfilled, OtcError::OfferAlreadyFulfilled);
        let clock = &ctx.accounts.clock;
        require!(clock.unix_timestamp > offer.expiration_ts, OtcError::OfferNotExpired);
        offer.fulfilled = true;
        emit!(OfferExpired {
            maker: offer.maker,
            expiration_ts: offer.expiration_ts,
        });
        Ok(())
    }

    pub fn add_partner(ctx: Context<ModifyPartner>, partner: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        require!((pool.partners.len() as u8) < pool.max_partners, OtcError::PartnerLimitReached);
        require!(!pool.partners.contains(&partner), OtcError::PartnerAlreadyExists);
        pool.partners.push(partner);
        emit!(PartnerAdded { partner });
        Ok(())
    }

    pub fn remove_partner(ctx: Context<ModifyPartner>, partner: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        if let Some(i) = pool.partners.iter().position(|x| *x == partner) {
            pool.partners.swap_remove(i);
            emit!(PartnerRemoved { partner });
            Ok(())
        } else {
            err!(OtcError::PartnerNotFound)
        }
    }

    pub fn add_supported_pair(
        ctx: Context<ModifySupportedPair>,
        mint_a: Pubkey,
        mint_b: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        require!(pool.whitelisted_mints.contains(&mint_a), OtcError::MintNotWhitelisted);
        require!(pool.whitelisted_mints.contains(&mint_b), OtcError::MintNotWhitelisted);
        require!(
            pool.supported_pairs.iter().all(|p| p.mint_a != mint_a || p.mint_b != mint_b),
            OtcError::PairAlreadyExists
        );
        pool.supported_pairs.push(Pair { mint_a, mint_b });
        emit!(SupportedPairAdded { mint_a, mint_b });
        Ok(())
    }

    pub fn remove_supported_pair(
        ctx: Context<ModifySupportedPair>,
        mint_a: Pubkey,
        mint_b: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        if let Some(i) = pool.supported_pairs.iter().position(|p| p.mint_a == mint_a && p.mint_b == mint_b) {
            pool.supported_pairs.swap_remove(i);
            emit!(SupportedPairRemoved { mint_a, mint_b });
            Ok(())
        } else {
            err!(OtcError::PairNotFound)
        }
    }

    pub fn pause_pool(ctx: Context<ModifyPoolState>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        pool.paused = true;
        emit!(PoolPaused {
            admin: pool.authority,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn resume_pool(ctx: Context<ModifyPoolState>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        pool.paused = false;
        emit!(PoolResumed {
            admin: pool.authority,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn swap_direct(
        ctx: Context<SwapDirect>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(!pool.paused, OtcError::PoolIsPaused);
        require!(
            amount_a >= pool.min_swap_amount && amount_b >= pool.min_swap_amount,
            OtcError::SwapBelowMinimum
        );

        let party_a = *ctx.accounts.party_a.key;
        let party_b = *ctx.accounts.party_b.key;
        require!(pool.partners.contains(&party_a), OtcError::UnauthorizedPartner);
        require!(pool.partners.contains(&party_b), OtcError::UnauthorizedPartner);

        let mint_a = ctx.accounts.party_a_token_src.mint;
        let mint_b = ctx.accounts.party_b_token_src.mint;
        require!(pool.whitelisted_mints.contains(&mint_a), OtcError::MintNotWhitelisted);
        require!(pool.whitelisted_mints.contains(&mint_b), OtcError::MintNotWhitelisted);
        require!(
            pool.supported_pairs.iter().any(|p| p.mint_a == mint_a && p.mint_b == mint_b),
            OtcError::PairNotSupported
        );

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.party_a_token_src.to_account_info(),
                    to: ctx.accounts.party_b_token_dest.to_account_info(),
                    authority: ctx.accounts.party_a.to_account_info(),
                },
            ),
            amount_a,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.party_b_token_src.to_account_info(),
                    to: ctx.accounts.party_a_token_dest.to_account_info(),
                    authority: ctx.accounts.party_b.to_account_info(),
                },
            ),
            amount_b,
        )?;

        emit!(SwapDirectExecuted {
            party_a,
            party_b,
            mint_a,
            mint_b,
            filled_amount_a: amount_a,
            filled_amount_b: amount_b,
            remaining_amount_a: amount_a,
            remaining_amount_b: amount_b,
        });
        Ok(())
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        amount_a: u64,
        amount_b: u64,
        expiration_ts: i64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(!pool.paused, OtcError::PoolIsPaused);
        require!(
            amount_a >= pool.min_swap_amount && amount_b >= pool.min_swap_amount,
            OtcError::SwapBelowMinimum
        );

        let clock = Clock::get()?;
        require!(
            expiration_ts <= clock.unix_timestamp + pool.max_expiration_secs,
            OtcError::ExpirationTooLong
        );

        let maker = *ctx.accounts.maker.key;
        require!(pool.partners.contains(&maker), OtcError::UnauthorizedPartner);

        let mint_a = ctx.accounts.mint_a.key();
        let mint_b = ctx.accounts.mint_b.key();
        require!(pool.whitelisted_mints.contains(&mint_a), OtcError::MintNotWhitelisted);
        require!(pool.whitelisted_mints.contains(&mint_b), OtcError::MintNotWhitelisted);
        require!(
            pool.supported_pairs.iter().any(|p| p.mint_a == mint_a && p.mint_b == mint_b),
            OtcError::PairNotSupported
        );

        let offer = &mut ctx.accounts.offer;
        offer.maker = maker;
        offer.mint_a = mint_a;
        offer.mint_b = mint_b;
        offer.original_amount_a = amount_a;
        offer.original_amount_b = amount_b;
        offer.amount_a = amount_a;
        offer.amount_b = amount_b;
        offer.expiration_ts = expiration_ts;
        offer.fulfilled = false;
        offer.bump = ctx.bumps.offer;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.maker_token_src.to_account_info(),
                    to: ctx.accounts.escrow_account.to_account_info(),
                    authority: ctx.accounts.maker.to_account_info(),
                },
            ),
            amount_a,
        )?;

        emit!(OfferCreated {
            maker,
            mint_a,
            mint_b,
            amount_a,
            amount_b,
            expiration_ts,
        });
        Ok(())
    }

  pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let maker = *ctx.accounts.maker.key;
    let offer_account = &ctx.accounts.offer;
    
    require_keys_eq!(offer_account.maker, maker, OtcError::Unauthorized);
    require!(!offer_account.fulfilled, OtcError::OfferAlreadyFulfilled);
    require!(ctx.accounts.clock.unix_timestamp <= offer_account.expiration_ts, OtcError::OfferExpired);

    let fee_amount = (offer_account.amount_a as u128)
        .checked_mul(pool.fee_bps as u128).unwrap()
        .checked_div(10_000).unwrap() as u64;
    let net = offer_account.amount_a.checked_sub(fee_amount).unwrap();

    let seeds = &[b"offer", offer_account.maker.as_ref(), &[offer_account.bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        fee_amount,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.maker_token_dest.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        net,
    )?;

    // Now get mutable reference after all immutable uses
    let offer = &mut ctx.accounts.offer;
    offer.fulfilled = true;
    emit!(OfferCancelled { maker });
    Ok(())
}

 pub fn accept_offer(ctx: Context<AcceptOffer>, fill_amount_b: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let offer_account = &ctx.accounts.offer;
    let taker = *ctx.accounts.taker.key;

    require!(!pool.paused, OtcError::PoolIsPaused);
    require!(!offer_account.fulfilled, OtcError::OfferAlreadyFulfilled);
    require!(ctx.accounts.clock.unix_timestamp <= offer_account.expiration_ts, OtcError::OfferExpired);
    require!(pool.partners.contains(&taker), OtcError::UnauthorizedPartner);

    require!(fill_amount_b > 0 && fill_amount_b <= offer_account.amount_b, OtcError::InvalidFillAmount);
    let take_b = fill_amount_b;
    let take_a = (offer_account.original_amount_a as u128)
        .checked_mul(take_b as u128).unwrap()
        .checked_div(offer_account.original_amount_b as u128).unwrap() as u64;
    require!(take_a <= offer_account.amount_a, OtcError::InvalidFillAmount);

    let fee_amount = (take_a as u128)
        .checked_mul(pool.fee_bps as u128).unwrap()
        .checked_div(10_000).unwrap() as u64;
    let net_a = take_a.checked_sub(fee_amount).unwrap();

    let seeds = &[b"offer", offer_account.maker.as_ref(), &[offer_account.bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.maker.to_account_info(),
                to: ctx.accounts.maker_token_dest.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        ),
        take_b,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        fee_amount,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.taker_token_dest.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        net_a,
    )?;

    // Now get mutable reference after all immutable uses
    let offer = &mut ctx.accounts.offer;
    offer.amount_b = offer.amount_b.checked_sub(take_b).unwrap();
    offer.amount_a = offer.amount_a.checked_sub(take_a).unwrap();
    if offer.amount_b == 0 {
        offer.fulfilled = true;
    }

    emit!(OfferExecuted {
        maker: offer.maker,
        taker,
        mint_a: offer.mint_a,
        mint_b: offer.mint_b,
        filled_amount_a: take_a,
        filled_amount_b: take_b,
        remaining_amount_a: offer.amount_a,
        remaining_amount_b: offer.amount_b,
    });
    Ok(())
}

    pub fn extend_offer(ctx: Context<ExtendOffer>, new_expiration_ts: i64) -> Result<()> {
        let offer = &mut ctx.accounts.offer;
        let maker = *ctx.accounts.maker.key;
        let clock = Clock::get()?;

        require_keys_eq!(offer.maker, maker, OtcError::Unauthorized);
        require!(!offer.fulfilled, OtcError::OfferAlreadyFulfilled);
        require!(new_expiration_ts > offer.expiration_ts, OtcError::InvalidExtension);
        require!(
            new_expiration_ts <= clock.unix_timestamp + ctx.accounts.pool.max_expiration_secs,
            OtcError::ExpirationTooLong
        );

        offer.expiration_ts = new_expiration_ts;
        emit!(OfferExtended { maker, new_expiration_ts });
        Ok(())
    }
}

/// ========== State & Events ==========

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub max_partners: u8,
    pub partners: Vec<Pubkey>,
    pub whitelisted_mints: Vec<Pubkey>,
    pub supported_pairs: Vec<Pair>,
    pub paused: bool,
    pub fee_bps: u16,
    pub treasury: Pubkey,
    pub min_swap_amount: u64,
    pub max_expiration_secs: i64,
}

impl Pool {
    pub const MAX_PARTNERS: usize = 255;
    pub const MAX_MINTS: usize = 10;
    pub const MAX_PAIRS: usize = 10;
    pub const LEN: usize = 8
        + 32
        + 1
        + 4 + 32 * Self::MAX_PARTNERS
        + 4 + 32 * Self::MAX_MINTS
        + 4 + Pair::LEN * Self::MAX_PAIRS
        + 1
        + 2
        + 32
        + 8
        + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Pair {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}

impl Pair {
    pub const LEN: usize = 32 + 32;
}

#[account]
pub struct Offer {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub original_amount_a: u64,
    pub original_amount_b: u64,
    pub amount_a: u64,
    pub amount_b: u64,
    pub expiration_ts: i64,
    pub fulfilled: bool,
    pub bump: u8,
}

impl Offer {
    pub const LEN: usize = 8
        + 32
        + 32
        + 32
        + 8
        + 8
        + 8
        + 8
        + 8
        + 1
        + 1;
}

#[event]
pub struct PoolInitialized {
    pub authority: Pubkey,
    pub max_partners: u8,
    pub fee_bps: u16,
    pub treasury: Pubkey,
    pub min_swap_amount: u64,
    pub max_expiration_secs: i64,
}

#[event]
pub struct AuthorityTransferred {
    pub previous: Pubkey,
    pub new: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub previous: Pubkey,
    pub new: Pubkey,
}

#[event]
pub struct MintWhitelisted {
    pub mint: Pubkey,
}

#[event]
pub struct MintRemoved {
    pub mint: Pubkey,
}

#[event]
pub struct OfferExpired {
    pub maker: Pubkey,
    pub expiration_ts: i64,
}

#[event]
pub struct PartnerAdded {
    pub partner: Pubkey,
}

#[event]
pub struct PartnerRemoved {
    pub partner: Pubkey,
}

#[event]
pub struct SupportedPairAdded {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}

#[event]
pub struct SupportedPairRemoved {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}

#[event]
pub struct PoolPaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PoolResumed {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SwapDirectExecuted {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub filled_amount_a: u64,
    pub filled_amount_b: u64,
    pub remaining_amount_a: u64,
    pub remaining_amount_b: u64,
}

#[event]
pub struct OfferCreated {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub expiration_ts: i64,
}

#[event]
pub struct OfferCancelled {
    pub maker: Pubkey,
}

#[event]
pub struct OfferExecuted {
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub filled_amount_a: u64,
    pub filled_amount_b: u64,
    pub remaining_amount_a: u64,
    pub remaining_amount_b: u64,
}

#[event]
pub struct OfferExtended {
    pub maker: Pubkey,
    pub new_expiration_ts: i64,
}

/// ========== Accounts Contexts ==========

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = authority, space = Pool::LEN)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyPoolMints<'info> {
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyPartner<'info> {
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifySupportedPair<'info> {
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyPoolState<'info> {
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SwapDirect<'info> {
    pub pool: Account<'info, Pool>,

    #[account(signer)]
    pub party_a: AccountInfo<'info>,
    #[account(signer)]
    pub party_b: AccountInfo<'info>,

    #[account(mut, token::authority = party_a)]
    pub party_a_token_src: Account<'info, TokenAccount>,
    #[account(mut, token::authority = party_b)]
    pub party_b_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = party_b)]
    pub party_b_token_src: Account<'info, TokenAccount>,
    #[account(mut, token::authority = party_a)]
    pub party_a_token_dest: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount_a: u64, amount_b: u64, expiration_ts: i64)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        payer = maker,
        space = Offer::LEN,
        seeds = [b"offer", maker.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(mut, token::authority = maker, token::mint = mint_a)]
    pub maker_token_src: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        token::authority = offer,
        token::mint = mint_a,
        seeds = [b"offer", maker.key().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut, has_one = maker)]
    pub offer: Account<'info, Offer>,

    #[account(mut, token::authority = offer, token::mint = offer.mint_a)]
    pub escrow_account: Account<'info, TokenAccount>,

    pub maker: Signer<'info>,

    #[account(mut, token::authority = maker, token::mint = offer.mint_a)]
    pub maker_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = pool.treasury, token::mint = offer.mint_a)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(fill_amount_b: u64)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut, has_one = maker)]
    pub offer: Account<'info, Offer>,

    /// Added to satisfy has_one = maker
    pub maker: AccountInfo<'info>,

    #[account(signer)]
    pub taker: AccountInfo<'info>,

    #[account(mut, token::authority = taker, token::mint = offer.mint_b)]
    pub taker_token_src: Account<'info, TokenAccount>,

    #[account(mut, token::authority = pool.treasury, token::mint = offer.mint_b)]
    pub maker_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = offer, token::mint = offer.mint_a)]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(mut, token::authority = taker, token::mint = offer.mint_a)]
    pub taker_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = pool.treasury, token::mint = offer.mint_a)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ExtendOffer<'info> {
    #[account(mut, has_one = maker)]
    pub offer: Account<'info, Offer>,
    #[account(signer)]
    pub maker: AccountInfo<'info>,
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct CloseExpiredOffer<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub offer: Account<'info, Offer>,
    pub clock: Sysvar<'info, Clock>,
}

#[error_code]
pub enum OtcError {
    #[msg("Not authorized")]
    Unauthorized,
    #[msg("Partner limit reached")]
    PartnerLimitReached,
    #[msg("Partner already exists")]
    PartnerAlreadyExists,
    #[msg("Partner not found")]
    PartnerNotFound,
    #[msg("Pair already exists")]
    PairAlreadyExists,
    #[msg("Pair not found")]
    PairNotFound,
    #[msg("Pair not supported")]
    PairNotSupported,
    #[msg("Pool is paused")]
    PoolIsPaused,
    #[msg("Caller is not whitelisted")]
    UnauthorizedPartner,
    #[msg("Offer already fulfilled")]
    OfferAlreadyFulfilled,
    #[msg("Offer expired")]
    OfferExpired,
    #[msg("Offer not yet expired")]
    OfferNotExpired,
    #[msg("Swap amount below minimum")]
    SwapBelowMinimum,
    #[msg("Invalid fill amount")]
    InvalidFillAmount,
    #[msg("Mint already whitelisted")]
    MintAlreadyWhitelisted,
    #[msg("Mint not whitelisted")]
    MintNotWhitelisted,
    #[msg("Invalid treasury account")]
    InvalidTreasuryAccount,
    #[msg("Invalid extension")]
    InvalidExtension,
    #[msg("Expiration too long")]
    ExpirationTooLong,
}

