use crate::states::{Config, Gig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseGig<'info> {
    #[account(
        mut,
        constraint = gig.pending_deals == 0,
    )]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        constraint = payer.key() == gig.payer.key(),
    )]
    /// CHECK: constraint to gig's payer
    pub payer: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = owner.key() == gig.owner.key() || owner.key() == config.authority.key(),
    )]
    pub owner: Signer<'info>,

    pub config: Account<'info, Config>,
}

pub fn close_gig_handler(ctx: Context<CloseGig>) -> Result<()> {
    let source_account_info = ctx.accounts.gig.to_account_info();
    let dest_account_info = ctx.accounts.payer.to_account_info();

    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    let mut source_data = source_account_info.data.borrow_mut();
    source_data.fill(0);

    Ok(())
}
