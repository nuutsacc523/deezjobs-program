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
    /// * 2 - Has current deal
    pub state: u8,

    /// Service being offered, eg. Web development, UX / UI Design, etc. (1)
    pub category: u8,

    /// The skills under the specified category. (8)
    pub skills: u64,

    /// The price of this Gig. (8)
    /// Note: decimal places depends on the mint.
    pub asking: u64,

    /// Should be less than the asking price. (8)
    /// Note: decimal places depends on the mint.
    pub min_accepted_offer: u64,

    /// The minimum deadline that the freelancer can offer. In seconds. (8)
    pub min_completion_time: u64,

    /// Address who covered the creation of this account. (32)
    /// This can be Deezjobs or the freelancer.
    pub payer: Pubkey,

    /// SPL token for the gig payment. (33)
    /// If set to None, the freelancer is expecting SOL as payment.
    pub mint: Option<Pubkey>,
}

impl Gig {
    pub fn len() -> usize {
        8 + 1 + 32 + 1 + 1 + 8 + 8 + 8 + 8 + 32 + 33
    }
}
