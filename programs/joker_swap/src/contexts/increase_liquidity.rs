use crate::states::{HybridPool, UserData};
use crate::utils::verify;
use crate::ErrorCode;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_spl::{
    self,
    token_interface::{self as token, Mint, TokenAccount, TokenInterface, TransferChecked},
};

pub fn increase_liquidity(
    ctx: Context<AddLiquidity>,
    _id: u16,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let hybrid_pool = &mut ctx.accounts.pool_account;
    let user_data = &mut ctx.accounts.user_data;
    if !user_data.initialized {
        user_data.initialized = true;
        user_data.owner = ctx.accounts.signer.key();
        user_data.pool = hybrid_pool.key();
        user_data.nft_count = 0;
        user_data.token_amount = 0;
    }
    let nft_mint_keccak = keccak::hash(ctx.accounts.nft_mint.key().as_ref());
    require!(
        verify(proof, hybrid_pool.root_mint, nft_mint_keccak.0),
        ErrorCode::InvalidProof
    );
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.spl_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.hybrid_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        hybrid_pool.nft_price,
        ctx.accounts.token_mint.decimals,
    )?;

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.nft_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_nft_account.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.hybrid_nft_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        1,
        ctx.accounts.nft_mint.decimals,
    )?;

    user_data.nft_count += 1;
    user_data.token_amount += hybrid_pool.nft_price;
    Ok(())
}

#[derive(Accounts)]
#[instruction(id:u16)]
pub struct AddLiquidity<'info> {
    #[account(constraint = pool_account.token_mint== token_mint.key()&& pool_account.id == id)]
    pub pool_account: Box<Account<'info, HybridPool>>,
    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut,seeds = [b"token-pool", pool_account.key().as_ref(), token_mint.key().as_ref()], bump)]
    pub hybrid_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(init_if_needed, payer = signer, seeds = [b"nft-pool", pool_account.key().as_ref(), nft_mint.key().as_ref()], bump, token::mint = nft_mint, token::authority = pool_account,token::token_program = nft_program)]
    pub hybrid_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(init_if_needed,space=64+33, payer = signer, seeds = [b"user-data",signer.key().as_ref() ,pool_account.key().as_ref()], bump)]
    pub user_data: Box<Account<'info, UserData>>,
    #[account(mut)]
    #[account(constraint = user_token_account.mint== token_mint.key())]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    #[account(constraint = user_nft_account.mint== nft_mint.key())]
    pub user_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub nft_program: Interface<'info, TokenInterface>,
    pub spl_program: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
