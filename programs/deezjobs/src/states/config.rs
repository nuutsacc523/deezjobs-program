use anchor_lang::prelude::*;

/// Config account serves as the global settings of the protocol.
#[account]
pub struct Config {
    /// Bump nonce of the PDA. (1)
    pub bump: u8,

    /// Super authority. (32)
    pub authority: Pubkey,

    /// Account which will hold all collected fee. (32)
    pub treasury: Pubkey,

    /// Fee to collect from the client for every accepted deal. Percentage with 2 decimal places (0 to 10000). (2)
    pub client_fee_percentage: u16,

    /// If client fee is less than this value, this value will serve as the fee (in USDC). 6 decimal places. (8)
    pub client_fee_min: u64,

    /// Fee to collect from the freelancer for each completed deal. Percentage with 2 decimal places (0 to 10000). (2)
    pub freelancer_fee_percentage: u16,

    /// Bounty of the referral when the deal is completed. Percentage with 2 decimal places (0 to 10000). (2)
    pub referral_fee_percentage: u16,
}

impl Config {
    pub fn len() -> usize {
        8 + 1 + 32 + 32 + 2 + 8 + 2 + 2
    }
}
