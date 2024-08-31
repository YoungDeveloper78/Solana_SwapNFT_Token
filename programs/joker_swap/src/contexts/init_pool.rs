use crate::states::{Config, HybridPool};
use anchor_lang::prelude::*;

use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
#[instruction(id: u16)]
pub struct InitializePool<'info> {
    #[account(init, payer = signer, space = 1024, seeds = [b"pool", signer.key().as_ref(), id.to_le_bytes().as_ref()], bump)]
    pub pool_account: Box<Account<'info, HybridPool>>, // TODO: why not just name it hybrid_pool_account?
    #[account(mut,seeds = [b"config"],bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(init, payer = signer, seeds = [b"token-pool", pool_account.key().as_ref(), token_mint.key().as_ref()], bump, token::mint = token_mint, token::authority = pool_account)]
    pub hybrid_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    /// these are checks for the fee recievers which was declared in the init_config instruction.
    #[account(mut)]
    #[account(constraint = fee_receiver1.key()== config.fee_receivers[0])]
    ///CHECK:not important
    pub fee_receiver1: AccountInfo<'info>,
    #[account(constraint = fee_receiver2.key()== config.fee_receivers[1])]
    #[account(mut)]
    ///CHECK:not important
    pub fee_receiver2: AccountInfo<'info>,
    #[account(constraint = fee_receiver3.key()== config.fee_receivers[2])]
    #[account(mut)]
    ///CHECK:not important
    pub fee_receiver3: AccountInfo<'info>,
    #[account(constraint = fee_receiver4.key()== config.fee_receivers[3])]
    #[account(mut)]
    ///CHECK:not important
    pub fee_receiver4: AccountInfo<'info>,
    #[account(constraint = fee_receiver5.key()== config.fee_receivers[4])]
    #[account(mut)]
    ///CHECK:not important
    pub fee_receiver5: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    id: u16,
    price: u64,
    root: [u8; 32],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let hybrid_pool = &mut ctx.accounts.pool_account;
    let fee = config.init_fee;

    let ix = anchor_lang::solana_program::system_instruction::transfer( // TODO: why not using system program instead? 
        &ctx.accounts.signer.key(),
        &ctx.accounts.fee_receiver1.key(),
        fee.to_owned() * config.fee_percentage[0] as u64 / 100,
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fee_receiver1.to_account_info(),
        ],
    )?;
    // ?
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &ctx.accounts.fee_receiver2.key(),
        fee.to_owned() * config.fee_percentage[1] as u64 / 100,
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fee_receiver2.to_account_info(),
        ],
    )?;

    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &ctx.accounts.fee_receiver3.key(),
        fee.to_owned() * config.fee_percentage[2] as u64 / 100,
    );
    // ?
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fee_receiver3.to_account_info(),
        ],
    )?;

    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &ctx.accounts.fee_receiver4.key(),
        fee.to_owned() * config.fee_percentage[3] as u64 / 100,
    );
    // ?
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fee_receiver4.to_account_info(),
        ],
    )?;

    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.signer.key(),
        &ctx.accounts.fee_receiver5.key(),
        fee.to_owned() * config.fee_percentage[4] as u64 / 100,
    );
    // ?
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.fee_receiver5.to_account_info(),
        ],
    )?;

    hybrid_pool.nft_price = price;
    hybrid_pool.token_mint = ctx.accounts.token_mint.key();
    hybrid_pool.root_mint = root;
    hybrid_pool.owner = ctx.accounts.signer.key();
    hybrid_pool.id = id;

    Ok(())
}
