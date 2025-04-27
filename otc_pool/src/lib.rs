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
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = *ctx.accounts.authority.key;
        pool.max_partners = max_partners;
        pool.partners = Vec::new();
        pool.supported_pairs = Vec::new();
        pool.paused = false;
        pool.fee_bps = fee_bps;
        pool.treasury = treasury;
        emit!(PoolInitialized {
            authority: pool.authority,
            max_partners,
            fee_bps,
            treasury,
        });
        Ok(())
    }

    pub fn add_partner(ctx: Context<ModifyPartner>, partner: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        require!(
            (pool.partners.len() as u8) < pool.max_partners,
            OtcError::PartnerLimitReached
        );
        require!(!pool.partners.contains(&partner), OtcError::PartnerAlreadyExists);
        pool.partners.push(partner);
        emit!(PartnerAdded { partner });
        Ok(())
    }

    pub fn remove_partner(ctx: Context<ModifyPartner>, partner: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        if let Some(idx) = pool.partners.iter().position(|x| *x == partner) {
            pool.partners.swap_remove(idx);
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
        require!(
            pool
                .supported_pairs
                .iter()
                .all(|p| !(p.mint_a == mint_a && p.mint_b == mint_b)),
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
        if let Some(idx) = pool
            .supported_pairs
            .iter()
            .position(|p| p.mint_a == mint_a && p.mint_b == mint_b)
        {
            pool.supported_pairs.swap_remove(idx);
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
        emit!(PoolPaused {});
        Ok(())
    }

    pub fn resume_pool(ctx: Context<ModifyPoolState>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(pool.authority, ctx.accounts.authority.key(), OtcError::Unauthorized);
        pool.paused = false;
        emit!(PoolResumed {});
        Ok(())
    }

    pub fn swap_direct(
        ctx: Context<SwapDirect>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(!pool.paused, OtcError::PoolIsPaused);

        let party_a = *ctx.accounts.party_a.key;
        let party_b = *ctx.accounts.party_b.key;

        require!(
            pool.partners.contains(&party_a),
            OtcError::UnauthorizedPartner
        );
        require!(
            pool.partners.contains(&party_b),
            OtcError::UnauthorizedPartner
        );

        let mint_a = ctx.accounts.party_a_token_src.mint;
        let mint_b = ctx.accounts.party_b_token_src.mint;
        require!(
            pool
                .supported_pairs
                .iter()
                .any(|p| p.mint_a == mint_a && p.mint_b == mint_b),
            OtcError::PairNotSupported
        );

        // Transfer A → B
        let cpi_a = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_a_token_src.to_account_info(),
                to: ctx.accounts.party_b_token_dest.to_account_info(),
                authority: ctx.accounts.party_a.to_account_info(),
            },
        );
        token::transfer(cpi_a, amount_a)?;

        // Transfer B → A
        let cpi_b = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.party_b_token_src.to_account_info(),
                to: ctx.accounts.party_a_token_dest.to_account_info(),
                authority: ctx.accounts.party_b.to_account_info(),
            },
        );
        token::transfer(cpi_b, amount_b)?;

        emit!(SwapDirectExecuted {
            party_a,
            party_b,
            mint_a,
            mint_b,
            amount_a,
            amount_b
        });
        Ok(())
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        amount_a: u64,
        amount_b: u64,
        expiration_ts: i64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(!pool.paused, OtcError::PoolIsPaused);

        let maker = *ctx.accounts.maker.key;
        require!(
            pool.partners.contains(&maker),
            OtcError::UnauthorizedPartner
        );

        let mint_a = ctx.accounts.mint_a.key();
        let mint_b = ctx.accounts.mint_b.key();
        require!(
            pool
                .supported_pairs
                .iter()
                .any(|p| p.mint_a == mint_a && p.mint_b == mint_b),
            OtcError::PairNotSupported
        );

        let offer = &mut ctx.accounts.offer;
        offer.maker = maker;
        offer.mint_a = mint_a;
        offer.mint_b = mint_b;
        offer.amount_a = amount_a;
        offer.amount_b = amount_b;
        offer.expiration_ts = expiration_ts;
        offer.fulfilled = false;
        offer.bump = ctx.bumps.offer;

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.maker_token_src.to_account_info(),
                to: ctx.accounts.escrow_account.to_account_info(),
                authority: ctx.accounts.maker.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount_a)?;

        emit!(OfferCreated {
            maker,
            mint_a,
            mint_b,
            amount_a,
            amount_b,
            expiration_ts
        });
        Ok(())
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        // take immutable AccountInfo first
        let offer_info = ctx.accounts.offer.to_account_info();
        let offer = &mut ctx.accounts.offer;
        let maker = *ctx.accounts.maker.key;
        require_keys_eq!(offer.maker, maker, OtcError::Unauthorized);
        require!(!offer.fulfilled, OtcError::OfferAlreadyFulfilled);

        // prepare seeds for PDA
        let seeds: &[&[u8]] = &[b"offer", offer.maker.as_ref(), &[offer.bump]];

        // perform refund from escrow → maker
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.maker_token_dest.to_account_info(),
                authority: offer_info.clone(),
            },
        );
        token::transfer(cpi_ctx.with_signer(&[seeds]), offer.amount_a)?;

        offer.fulfilled = true;
        emit!(OfferCancelled { maker });
        Ok(())
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        // take immutable AccountInfo first
        let offer_info = ctx.accounts.offer.to_account_info();
        let pool = &ctx.accounts.pool;
        require!(!pool.paused, OtcError::PoolIsPaused);

        let offer = &mut ctx.accounts.offer;
        let taker = *ctx.accounts.taker.key;
        require!(
            pool.partners.contains(&taker),
            OtcError::UnauthorizedPartner
        );
        require!(!offer.fulfilled, OtcError::OfferAlreadyFulfilled);

        let clock = &ctx.accounts.clock;
        require!(
            clock.unix_timestamp <= offer.expiration_ts,
            OtcError::OfferExpired
        );

        // taker → maker (Token B)
        let cpi_b = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.taker_token_src.to_account_info(),
                to: ctx.accounts.maker_token_dest.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        );
        token::transfer(cpi_b, offer.amount_b)?;

        // compute fee & remainder
        let fee_amount = (offer.amount_a as u128)
            .checked_mul(pool.fee_bps as u128)
            .unwrap()
            .checked_div(10_000)
            .unwrap() as u64;
        let amount_after_fee = offer.amount_a.checked_sub(fee_amount).unwrap();

        // seeds for PDA authority
        let seeds: &[&[u8]] = &[b"offer", offer.maker.as_ref(), &[offer.bump]];

        // escrow → treasury (fee)
        let cpi_fee = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: offer_info.clone(),
            },
        );
        token::transfer(cpi_fee.with_signer(&[seeds]), fee_amount)?;

        // escrow → taker (remaining Token A)
        let cpi_a = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.taker_token_dest.to_account_info(),
                authority: offer_info,
            },
        );
        token::transfer(cpi_a.with_signer(&[seeds]), amount_after_fee)?;

        offer.fulfilled = true;
        emit!(OfferExecuted {
            maker: offer.maker,
            taker,
            mint_a: offer.mint_a,
            mint_b: offer.mint_b,
            amount_a: offer.amount_a,
            amount_b: offer.amount_b
        });
        Ok(())
    }
}

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub max_partners: u8,
    pub partners: Vec<Pubkey>,
    pub supported_pairs: Vec<Pair>,
    pub paused: bool,
    pub fee_bps: u16,
    pub treasury: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Pair {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}

#[account]
pub struct Offer {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub expiration_ts: i64,
    pub fulfilled: bool,
    pub bump: u8,
}

/// Events

#[event]
pub struct PoolInitialized {
    pub authority: Pubkey,
    pub max_partners: u8,
    pub fee_bps: u16,
    pub treasury: Pubkey,
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
pub struct PoolPaused {}

#[event]
pub struct PoolResumed {}

#[event]
pub struct SwapDirectExecuted {
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
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
    pub amount_a: u64,
    pub amount_b: u64,
}

/// Account Contexts

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
    pub system_program: Program<'info, System>,
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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
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

    #[account(signer)]
    pub maker: AccountInfo<'info>,

    #[account(mut, token::authority = maker, token::mint = offer.mint_a)]
    pub maker_token_dest: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut, has_one = maker)]
    pub offer: Account<'info, Offer>,

    #[account(signer)]
    pub maker: AccountInfo<'info>,
    #[account(signer)]
    pub taker: AccountInfo<'info>,

    #[account(mut, token::authority = taker, token::mint = offer.mint_b)]
    pub taker_token_src: Account<'info, TokenAccount>,
    #[account(mut, token::authority = maker, token::mint = offer.mint_b)]
    pub maker_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = offer, token::mint = offer.mint_a)]
    pub escrow_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = taker, token::mint = offer.mint_a)]
    pub taker_token_dest: Account<'info, TokenAccount>,

    #[account(mut, token::authority = pool.treasury, token::mint = offer.mint_a)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Size constants

impl Pool {
    pub const MAX_PARTNERS: usize = 255;
    pub const MAX_PAIRS: usize = 10;
    pub const LEN: usize = 8
        + 32
        + 1
        + 4
        + (32 * Self::MAX_PARTNERS)
        + 4
        + (Pair::LEN * Self::MAX_PAIRS)
        + 1
        + 2
        + 32;
}

impl Pair {
    pub const LEN: usize = 32 + 32;
}

impl Offer {
    pub const LEN: usize = 8 + 96 + 16 + 8 + 1 + 1;
}

/// Errors

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
}
