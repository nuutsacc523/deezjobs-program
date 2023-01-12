use anchor_lang::{prelude::*, solana_program::clock};

use crate::states::{Deal, Gig};

#[derive(Accounts)]
pub struct AcceptDeal<'info> {
    #[account(
        mut,
        constraint = gig.owner.key() == freelancer.key(),
    )]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        constraint = deal.gig.key() == gig.key(),
        constraint = deal.state == 1,
    )]
    pub deal: Account<'info, Deal>,

    pub freelancer: Signer<'info>,
}

pub fn accept_deal_handler(ctx: Context<AcceptDeal>) -> Result<()> {
    let clock = clock::Clock::get()?;
    let gig = &mut ctx.accounts.gig;
    let deal = &mut ctx.accounts.deal;

    gig.pending_deals += 1;
    deal.time_accepted = clock.unix_timestamp;
    deal.state = 3;

    Ok(())
}
