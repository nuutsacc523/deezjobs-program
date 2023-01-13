use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer}, associated_token::AssociatedToken};

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
    /// CHECK:
    pub client: UncheckedAccount<'info>,

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

    #[account(
        mut,
        constraint = signer.key() == client.key() || signer.key() == config.authority.key(),
    )]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn complete_deal_handler(ctx: Context<CompleteDeal>) -> Result<()> {

    let gig = &mut ctx.accounts.gig;
    let deal = &mut ctx.accounts.deal;
    let client = &mut ctx.accounts.client;
    let config = &ctx.accounts.config;

    let escrow = &mut ctx.accounts.escrow;
    let freelancer_token_account = &mut ctx.accounts.freelancer_token_account;
    let referrer_token_account = &mut ctx.accounts.referrer_token_account;
    let treasury_token_account = &mut ctx.accounts.treasury_token_account;

    gig.pending_deals -= 1;
    deal.state |= 8;

    // Compute fees.

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

    let freelancer_fee_percentage: u64 = config.freelancer_fee_percentage.try_into().unwrap();    

    let freelancer_fee = freelancer_fee_percentage
        .checked_mul(deal.offer)
        .unwrap()
        .checked_div(100_00)
        .unwrap();

    let referral_pay = match referrer_token_account {
        Some(_) => {
            let referral_fee_percentage: u64 = config.referral_fee_percentage.try_into().unwrap();    

            referral_fee_percentage
                .checked_mul(deal.offer)
                .unwrap()
                .checked_div(100_00)
                .unwrap()
        },
        None => 0,
    };

    let freelancer_pay = deal.offer - freelancer_fee - referral_pay;
    let treasury_pay = escrow.amount - freelancer_pay - referral_pay;

    // Transfer to freelancer.

    let transfer_ix = Transfer {
        from: escrow.to_account_info(),
        to: freelancer_token_account.to_account_info(),
        authority: deal.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_ix,
        deal_sig.as_slice(),
    );

    anchor_spl::token::transfer(cpi_ctx, freelancer_pay)?;

    // Transfer to referrer.

    match referrer_token_account {
        Some(referrer_token_account) => {
            let transfer_ix = Transfer {
                from: escrow.to_account_info(),
                to: referrer_token_account.to_account_info(),
                authority: deal.to_account_info(),
            };
        
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_ix,
                deal_sig.as_slice(),
            );
        
            anchor_spl::token::transfer(cpi_ctx, referral_pay)?;
        },
        None => (),
    }

    // Transfer to treasury.
    // Remainder of the escrow amount, this should include the client's fee as well.

    let transfer_ix = Transfer {
        from: escrow.to_account_info(),
        to: treasury_token_account.to_account_info(),
        authority: deal.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_ix,
        deal_sig.as_slice(),
    );

    anchor_spl::token::transfer(cpi_ctx, treasury_pay)?;
    
    // Close escrow account, give back the rent to client.

    let source_account_info = escrow.to_account_info();
    let dest_account_info = client.to_account_info();

    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    let mut source_data = source_account_info.data.borrow_mut();
    source_data.fill(0);

    Ok(())
}