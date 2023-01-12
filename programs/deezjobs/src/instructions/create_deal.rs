use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

use crate::{
    states::{Config, Deal, Gig},
    CustomError,
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateDealParams {
    pub referrer: Option<Pubkey>,
    pub offer: u64,
    pub deadline: i64,
}

#[derive(Accounts)]
#[instruction(params: CreateDealParams)]
pub struct CreateDeal<'info> {
    #[account(
        init,
        payer = owner,
        seeds = [
            b"deal",
            owner.key().as_ref(),
            gig.key().as_ref(),
        ],
        bump,
        space = Deal::len()
    )]
    pub deal: Account<'info, Deal>,

    #[account(
        init,
        payer = owner,
        seeds = [
            b"deal_escrow",
            deal.key().as_ref(),
        ],
        bump,
        token::mint = mint,
        token::authority = deal,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_wallet.owner == owner.key(),
        constraint = owner_wallet.mint == mint.key(),
    )]
    pub owner_wallet: Account<'info, TokenAccount>,

    #[account(
        // TODO: will fail for native token
        // Solution: possibly put every account involved to optional
        constraint = mint.key() == gig.mint.unwrap().key(),
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = gig.state == 1,
        constraint = gig.asking <= params.offer @ CustomError::InsufficientOffer,
    )]
    pub gig: Account<'info, Gig>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn create_deal_handler(ctx: Context<CreateDeal>, params: CreateDealParams) -> Result<()> {
    let deal = &mut ctx.accounts.deal;
    let escrow = &mut ctx.accounts.escrow;
    let owner_wallet = &mut ctx.accounts.owner_wallet;
    let gig = &ctx.accounts.gig;
    let config = &ctx.accounts.config;
    let client = &ctx.accounts.owner;
    let clock = clock::Clock::get()?;

    let client_fee_percentage: u64 = config.client_fee_percentage.try_into().unwrap();

    let client_fee = client_fee_percentage
        .checked_mul(params.offer)
        .unwrap()
        .checked_div(100_00)
        .unwrap();

    // TODO: client_fee_min is assumed to be USDC at the moment, possible source of bug
    let client_fee = if client_fee < config.client_fee_min {
        config.client_fee_min
    } else {
        client_fee
    };

    let total_escrow_amount = params.offer + client_fee;

    let transfer_ix = Transfer {
        from: owner_wallet.to_account_info(),
        to: escrow.to_account_info(),
        authority: client.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_ix);

    anchor_spl::token::transfer(cpi_ctx, u64::from(total_escrow_amount))?;

    deal.bump = *ctx.bumps.get("deal").unwrap();
    deal.escrow_bump = *ctx.bumps.get("escrow").unwrap();
    deal.offer = params.offer;
    deal.state = 1;
    deal.gig = gig.key();
    deal.freelancer = gig.owner.key();
    deal.client = client.key();
    deal.time_created = clock.unix_timestamp;
    deal.deadline = params.deadline;
    // TODO: referrer could be the client itself, exploiting the pay
    deal.referrer = params.referrer;

    if deal.time_created + gig.min_completion_time > params.deadline {
        return Err(error!(CustomError::DeadlineTooShort));
    }

    Ok(())
}
