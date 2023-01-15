use anchor_lang::prelude::*;

pub mod instructions;
pub mod states;

pub use instructions::*;

declare_id!("A55j3MBSRHrKvpgACjSCAdgrQ4cQ5HsFf2omaRsiSHcU");

#[program]
pub mod deezjobs {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        initialize_handler(ctx, params)
    }

    pub fn create_gig(ctx: Context<CreateGig>, params: CreateGigParams) -> Result<()> {
        create_gig_handler(ctx, params)
    }

    pub fn close_gig(ctx: Context<CloseGig>) -> Result<()> {
        close_gig_handler(ctx)
    }

    pub fn create_deal(ctx: Context<CreateDeal>, params: CreateDealParams) -> Result<()> {
        create_deal_handler(ctx, params)
    }

    pub fn close_deal(ctx: Context<CloseDeal>) -> Result<()> {
        close_deal_handler(ctx)
    }

    pub fn accept_deal(ctx: Context<AcceptDeal>) -> Result<()> {
        accept_deal_handler(ctx)
    }

    pub fn complete_deal(ctx: Context<CompleteDeal>) -> Result<()> {
        complete_deal_handler(ctx)
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient offer amount")]
    InsufficientOffer,

    #[msg("Deadline is not long enough")]
    DeadlineTooShort,
}
