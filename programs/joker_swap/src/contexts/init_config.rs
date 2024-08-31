use crate::states::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::solana_zk_token_sdk::curve25519::scalar::Zeroable;
#[derive(Accounts)]
#[instruction(bump:u8)]
pub struct InitializeConfig<'info> {
    // TODO: it would be better if we add signer as a parametr to the seeds as well
    #[account(init_if_needed,space=256,payer=signer,seeds = [b"config"], bump)]
    #[account(constraint = config.owner== signer.key()||config.owner==Pubkey::zeroed())]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
pub fn initialize_config(
    ctx: Context<InitializeConfig>,
    _bump: u8,
    swap_fee: u64,
    init_fee: u64,
    fee_receivers: [Pubkey; 5],
    fee_percentage: [u8; 5],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.owner = ctx.accounts.signer.key();
    config.swap_fee = swap_fee;
    config.init_fee = init_fee;
    config.fee_receivers = fee_receivers;
    config.fee_percentage = fee_percentage;

    Ok(())
}
