use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token}, associated_token::AssociatedToken};

use crate::states::{Config, Gig, Deal};

#[derive(Accounts)]
pub struct CompleteDeal<'info> {
    #[account(
        mut,
        constraint = gig.key() == deal.gig.key(),
    )]
    pub gig: Box<Account<'info, Gig>>,

    #[account(
        mut,
        constraint = deal.state & 2 == 2,
    )]
    pub deal: Box<Account<'info, Deal>>,

    #[account(
        constraint = freelancer.key() == gig.owner.key(),
    )]
    /// CHECK:
    pub freelancer: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = client.key() == deal.client.key(),
    )]
    pub client: Signer<'info>,

    #[account(
        constraint = mint.key() == gig.mint.unwrap().key(),
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            b"deal_escrow",
            deal.key().as_ref(),
        ],
        bump = deal.escrow_bump,
        constraint = escrow.owner == deal.key(),
        constraint = escrow.mint == mint.key(),
    )]
    pub escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = client, 
        associated_token::mint = mint, 
        associated_token::authority = freelancer,
    )]
    pub freelancer_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = referrer.key() == deal.referrer.unwrap().key(),
    )]
    pub referrer: Option<UncheckedAccount<'info>>,

    #[account(
        init_if_needed,
        payer = client, 
        associated_token::mint = mint, 
        associated_token::authority = referrer,
    )]
    pub referrer_token_account: Option<Box<Account<'info, TokenAccount>>>,

    #[account(
        constraint = treasury.key() == config.treasury.key(),
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint, 
        associated_token::authority = treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, Config>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

// distribute escrow funds (to freelancer, to referrer, remainder to treasury)
// close escrow account (retain deal account for history purposes)
pub fn complete_deal_handler(ctx: Context<CompleteDeal>) -> Result<()> {

    let gig = &mut ctx.accounts.gig;
    let deal = &mut ctx.accounts.deal;
    let client = &ctx.accounts.client;
    let config = &ctx.accounts.config;

    let escrow = &mut ctx.accounts.escrow;
    let freelancer_token_account = &mut ctx.accounts.freelancer_token_account;
    let referrer_token_account = &mut ctx.accounts.referrer_token_account;
    let treasury_token_account = &mut ctx.accounts.treasury_token_account;

    gig.pending_deals -= 1;
    deal.state |= 8;

    // Compute pay less fees

    let gig_key = gig.key();
    let client_key = client.key();
    let deal_bump = deal.bump.to_le_bytes();

    let inner = vec![
        b"deal".as_ref(),
        client_key.as_ref(),
        gig_key.as_ref(),
        deal_bump.as_ref(),
    ];

    let deal_sig = vec![inner.as_slice()];

    // let transfer_ix = Transfer {
    //     from: escrow.to_account_info(),
    //     to: client_wallet.to_account_info(),
    //     authority: deal.to_account_info(),
    // };

    // let cpi_ctx = CpiContext::new_with_signer(
    //     ctx.accounts.token_program.to_account_info(),
    //     transfer_ix,
    //     deal_sig.as_slice(),
    // );

    // anchor_spl::token::transfer(cpi_ctx, escrow.amount)?;

    Ok(())
}