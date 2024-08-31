use anchor_lang::prelude::*;

pub mod contexts;
use contexts::*;
pub mod errors;
pub mod states;
pub mod utils;
pub use errors::ErrorCode;
pub mod pnft;


declare_id!("9xa8bCxxeeuVM953qj2zHQ2LLqF6YZPKo3Kmrk4HfXzS");

#[program]
pub mod neptune { 
    use super::*;
    /// this instruction is to initialize configurations for whole smart contract
    /// such as swap fees, initial fee, fee recievers and their percentages. 
    pub fn init_config(
        ctx: Context<InitializeConfig>,
        bump: u8,
        swap_fee: u64,
        init_fee: u64,
        fee_receivers: [Pubkey; 5],
        fee_percentage: [u8; 5],
    ) -> Result<()> {
        contexts::initialize_config(ctx, bump, swap_fee, init_fee, fee_receivers, fee_percentage)?;
        Ok(())
    }
    /// this instruction will intialize a pool each time we call it.
    /// it takes 3 parametrs:
    /// id: a unique identifier for the pool which is spicific to that pool only and can't be duplicated.
    /// price: this would be the price of the liquidity in the pool.
    /// root: this is the hash root of all nfts that is going to be in this pool.
    /// hash root will be created by hashing all the nfts hashes into one hash root.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        id: u16,
        price: u64,
        root: [u8; 32],
    ) -> Result<()> {
        contexts::initialize_pool(ctx, id, price, root)?;
        Ok(())
    }
    // TODO: why this instruction isn't implemented inside each add luqidity or remove liquidity?
    /// this instruction will change the hash root that was given to the smart contract when pool was initialized.
    /// it serves when you want to add or remove liquidity of nfts from the pool which will change the hash root of it and you need to
    /// set it to the new hash root.
    pub fn change_root(ctx: Context<ChangeRoot>, id: u16, bump: u8, root: [u8; 32]) -> Result<()> {
        contexts::change_root(ctx, id, bump, root)?;
        Ok(())
    }
    /// this instruction will add liquidity to the pool 
    pub fn add_liquidity(ctx: Context<AddLiquidity>, _id: u16, proof: Vec<[u8; 32]>) -> Result<()> {
        contexts::increase_liquidity(ctx, _id, proof)?;
        Ok(())
    }
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        id: u16,
        bump: u8,
        nft_count: u64,
    ) -> Result<()> {
        contexts::decrease_liquidity(ctx, id, bump, nft_count)?;
        Ok(())
    }

    pub fn swap_token_to_nft(
        ctx: Context<Swap>,
        id: u16,
        bump: u8,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        contexts::token_to_nft(ctx, id, bump, proof)?;
        Ok(())
    }

    pub fn swap_nft_to_token(
        ctx: Context<Swap>,
        id: u16,
        bump: u8,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        contexts::nft_to_token(ctx, id, bump, proof)?;
        Ok(())
    }
    // TODO-3 this function
    pub fn add_liquidity_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, IncreaseLiquidityPNFT<'info>>,
        id: u16,
        proof: Vec<[u8; 32]>,
        authorization_data: Option<AuthorizationDataLocal>,
        rules_acc_present: bool,
    ) -> Result<()> {
        contexts::increase_liquidity_pnft(ctx, id, proof, authorization_data, rules_acc_present)?;
        Ok(())
    }
    // TODO : 4 this function must be test
    pub fn swap_pnft_to_token<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapPnftToToken<'info>>,
        id: u16,
        bump: u8,
        proof: Vec<[u8; 32]>,
        authorization_data: Option<AuthorizationDataLocal>,
        rules_acc_present: bool,
    ) -> Result<()> {
        contexts::pnft_to_token(ctx, id, bump, proof, authorization_data, rules_acc_present)?;
        Ok(())
    }
    // TODO : 5x this function must be test

    pub fn swap_token_to_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapTokenToPNFT<'info>>,
        id: u16,
        bump: u8,
        proof: Vec<[u8; 32]>,
        authorization_data: Option<AuthorizationDataLocal>,
        rules_acc_present: bool,
    ) -> Result<()> {
        contexts::token_to_pnft(ctx, id, bump, proof, authorization_data, rules_acc_present)?;
        Ok(())
    }

    pub fn remove_liquidity_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, DecreaseiquidityPNFT<'info>>,
        id: u16,
        bump: u8,
        nft_count: u64,
        authorization_data: Option<AuthorizationDataLocal>,
        rules_acc_present: bool,
    ) -> Result<()> {
        contexts::decrease_liquidity_pnft(
            ctx,
            id,
            bump,
            nft_count,
            authorization_data,
            rules_acc_present,
        )?;
        Ok(())
    }

    pub fn test<'info>(ctx: Context<DeploymentTest>) -> Result<()> {
        msg!("deploy test v3");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct DeploymentTest<> {}