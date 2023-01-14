use crate::states::Gig;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateGigParams {
    pub category: u8,
    pub skills: u64,
    pub asking: u64,
    pub min_completion_time: i64,
}

#[derive(Accounts)]
#[instruction(params: CreateGigParams)]
pub struct CreateGig<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [
            b"gig",
            owner.key().as_ref(),
            &id.key().to_bytes()[..8],
        ],
        bump,
        space = Gig::len()
    )]
    pub gig: Account<'info, Gig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub id: Signer<'info>,

    pub mint: Option<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

pub fn create_gig_handler(ctx: Context<CreateGig>, params: CreateGigParams) -> Result<()> {
    let gig = &mut ctx.accounts.gig;
    gig.bump = *ctx.bumps.get("gig").unwrap();
    gig.nonce = ctx.accounts.id.key().to_bytes()[..8].try_into().unwrap();
    gig.payer = ctx.accounts.payer.key();
    gig.owner = ctx.accounts.owner.key();
    gig.mint = match &ctx.accounts.mint {
        Some(mint) => Some(mint.key()),
        None => None,
    };

    gig.state = 1; // if params.is_published { 1 } else { 0 };
    gig.pending_deals = 0;
    gig.category = params.category;
    gig.skills = params.skills;
    gig.asking = params.asking;
    gig.min_completion_time = params.min_completion_time;

    Ok(())
}
