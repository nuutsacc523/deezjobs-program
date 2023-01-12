use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

use crate::states::{Deal, Gig};

#[derive(Accounts)]
pub struct CloseDeal<'info> {
    #[account(
        mut,
        constraint = deal.state & 2 != 2,
    )]
    pub deal: Account<'info, Deal>,

    #[account(
        constraint = gig.key() == deal.gig.key(),
    )]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        constraint = client.key() == deal.client.key()
    )]
    /// CHECK: should be the owner of the deal
    pub client: UncheckedAccount<'info>,

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
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_wallet.owner == client.key(),
        constraint = owner_wallet.mint == mint.key(),
    )]
    pub owner_wallet: Account<'info, TokenAccount>,

    #[account(
        constraint = mint.key() == gig.mint.unwrap().key(),
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = signer.key() == deal.client.key() || signer.key() == gig.owner.key(),
    )]
    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn close_deal_handler(ctx: Context<CloseDeal>) -> Result<()> {
    let client = &mut ctx.accounts.client;
    let deal = &mut ctx.accounts.deal;
    let client_wallet = &mut ctx.accounts.owner_wallet;
    let escrow = &mut ctx.accounts.escrow;
    let gig = &ctx.accounts.gig;

    // Transfer escrow funds back to the client

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

    let transfer_ix = Transfer {
        from: escrow.to_account_info(),
        to: client_wallet.to_account_info(),
        authority: deal.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_ix,
        deal_sig.as_slice(),
    );

    anchor_spl::token::transfer(cpi_ctx, escrow.amount)?;

    // Close escrow account

    let source_account_info = escrow.to_account_info();
    let dest_account_info = client.to_account_info();

    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    let mut source_data = source_account_info.data.borrow_mut();
    source_data.fill(0);

    // Close deal account

    let source_account_info = deal.to_account_info();

    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    let mut source_data = source_account_info.data.borrow_mut();
    source_data.fill(0);

    Ok(())
}
