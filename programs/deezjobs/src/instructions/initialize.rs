use crate::{program::Deezjobs, states::Config};
use anchor_lang::prelude::*;
use anchor_spl::{token::{Token, TokenAccount, Mint}, associated_token::AssociatedToken};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeParams {
    pub client_fee_percentage: u16,
    pub client_fee_min: u64, // USDC
    pub freelancer_fee_percentage: u16,
    pub referral_fee_percentage: u16,
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = upgrade_authority,
        seeds = [b"config"],
        bump,
        space = Config::len()
    )]
    pub config: Account<'info, Config>,

    /// CHECK: 
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = upgrade_authority, 
        associated_token::mint = mint, 
        associated_token::authority = treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    // Should be USDC mint address
    pub mint: Account<'info, Mint>,

    #[account(
        constraint = program.programdata_address()? == Some(program_data.key())
    )]
    pub program: Program<'info, Deezjobs>,

    #[account(
        constraint = program_data.upgrade_authority_address == Some(upgrade_authority.key())
    )]
    pub program_data: Account<'info, ProgramData>,

    #[account(mut)]
    pub upgrade_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.bump = *ctx.bumps.get("config").unwrap();

    config.authority = ctx.accounts.upgrade_authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.client_fee_percentage = params.client_fee_percentage;
    config.client_fee_min = params.client_fee_min;
    config.freelancer_fee_percentage = params.freelancer_fee_percentage;
    config.referral_fee_percentage = params.referral_fee_percentage;

    Ok(())
}
