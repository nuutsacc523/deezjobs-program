use anchor_lang::prelude::*;

/// Gig is an account created by the freelancer which tells the details of the offered service.
#[account]
pub struct Gig {
    /// Bump nonce of the PDA. (1)
    pub bump: u8,

    /// Freelancer who owns this Gig. (32)
    pub owner: Pubkey,

    /// State of this Gig. (1)
    /// * 1 - Published
    pub state: u8,

    /// How many deals are currently accepted and ongoing. (1)
    /// Increments when user accepts a Deal.
    /// Decrements when a Deal for this Gig is settled / completed.
    pub pending_deals: u8,

    /// Service being offered, eg. Web development, UX / UI Design, etc. (1)
    pub category: u8,

    /// The skills under the specified category. (8)
    pub skills: u64,

    /// The minimum pay allowed of this Gig. (8)
    /// Note: decimal places depends on the mint.
    pub asking: u64,

    /// The minimum deadline that the freelancer can offer. In seconds. (8)
    pub min_completion_time: i64,

    /// Address who covered the creation of this account. (32)
    /// This can be Deezjobs or the owner of this Gig.
    pub payer: Pubkey,

    /// Random seed sliced from a pubkey, serves as account seed. (8)
    pub nonce: [u8; 8],

    /// SPL token for the gig payment. (33)
    /// If set to None, the freelancer is expecting SOL as payment.
    pub mint: Option<Pubkey>,
}

impl Gig {
    pub fn len() -> usize {
        8 + 1 + 32 + 1 + 1 + 1 + 8 + 8 + 8 + 32 + 8 + 33
    }
}
