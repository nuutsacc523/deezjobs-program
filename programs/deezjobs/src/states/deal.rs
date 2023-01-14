use anchor_lang::prelude::*;

/// A Deal is an account created by the client, related to the Gig created by the freelancer.
#[account]
pub struct Deal {
    /// Bump nonce of the PDA. (1)
    pub bump: u8,

    /// Owner of this deal. (32)
    pub client: Pubkey,

    /// State of this Gig. (1)
    /// * 1  - Published
    /// * 2  - Accepted by the freelancer
    /// * 4  - Has dispute
    /// * 8  - Completed / settled
    pub state: u8,

    /// The owner of the Gig. (32)
    pub freelancer: Pubkey,

    /// The Gig account. (32)
    pub gig: Pubkey,

    /// Agreed offer amount. (8)
    pub offer: u64,

    /// Gig must be completed before this deadline. Unix timestamp. (8)
    pub deadline: i64,

    /// Time this deal was created. (8)
    pub time_created: i64,

    /// Time when the freelancer accepted the offer. Unix timestamp. (8)
    pub time_accepted: i64,

    /// Address who referred the gig to the client. (33)
    pub referrer: Option<Pubkey>,
}

impl Deal {
    pub fn len() -> usize {
        8 + 1 + 32 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 33
    }
}
