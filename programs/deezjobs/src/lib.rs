use anchor_lang::prelude::*;

pub mod instructions;
pub mod states;

pub use instructions::*;

declare_id!("FjTokNmnP9MUJhGYj2DRcXoVwVq3utSceieoA3roKX7g");

#[program]
pub mod deezjobs {
    use super::*;

    // pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    //     Ok(())
    // }
}
