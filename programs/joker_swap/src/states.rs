use anchor_lang::prelude::*;

#[derive(Debug)] // TODO: why we using debug here? 
#[account]
pub struct Config {
    pub owner: Pubkey,
    pub fee_receivers: [Pubkey; 5],
    pub fee_percentage: [u8; 5],
    pub swap_fee: u64,
    pub init_fee: u64,
}

#[account]
pub struct HybridPool {
    pub owner: Pubkey,
    pub root_mint: [u8; 32],
    pub token_mint: Pubkey,
    pub nft_price: u64,
    pub id: u16,
}
#[account]
pub struct UserData {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub token_amount: u64,
    pub nft_count: u64,
    pub initialized: bool,
}

#[account]
pub struct PnftTokenAccountAddress {
    pub token_account: Pubkey,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct AllowList {
    pub proof: Vec<[u8; 32]>,
    pub current: u32,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ContainerParams {
    /// The program ID that is requesting randomness. Can be used for a generic function.
    pub program_id: Pubkey,
    /// The maximum guess the user submitted to bound the result from 1 to max_guess.
    pub max_guess: u8,
    /// The UserState account that submitted the guess.
    pub user_key: Pubkey,
}
