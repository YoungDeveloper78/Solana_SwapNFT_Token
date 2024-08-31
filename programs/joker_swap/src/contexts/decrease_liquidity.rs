use crate::states::{HybridPool, UserData};

use crate::ErrorCode;

use anchor_lang::prelude::*;

use anchor_spl::{
    self,
    token_interface::{self as token, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(id:u16)]
pub struct RemoveLiquidity<'info> {
    #[account(mut,constraint = pool_account.token_mint== token_mint.key()&& pool_account.id == id)]
    pub pool_account: Box<Account<'info, HybridPool>>,
    #[account(mut,seeds = [b"user-data",signer.key().as_ref(),pool_account.key().as_ref()], bump)]
    pub user_data: Box<Account<'info, UserData>>,
    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, seeds = [b"token-pool", pool_account.key().as_ref(), token_mint.key().as_ref()], bump)]
    pub hybrid_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, seeds = [b"nft-pool", pool_account.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub hybrid_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    #[account(constraint = user_token_account.mint== token_mint.key())]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    #[account(constraint = hybrid_nft_account.mint== nft_mint.key())]
    pub user_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub nft_program: Interface<'info, TokenInterface>,
    pub spl_program: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn decrease_liquidity(
    ctx: Context<RemoveLiquidity>,
    id: u16,
    bump: u8,
    nft_count: u64,
) -> Result<()> {
    let hybrid_pool = &mut ctx.accounts.pool_account;
    let user_data = &mut ctx.accounts.user_data;
    let owner = hybrid_pool.owner.key();
    let id = id.to_le_bytes();
    let bump_vector = bump.to_le_bytes();
    let inner = vec![b"pool".as_ref(), owner.as_ref(), &id, &bump_vector];
    let outer = vec![inner.as_slice()];
    let pool_balance = ctx.accounts.hybrid_token_account.amount;
    if nft_count > 0 && user_data.nft_count > 0 {
        let nft_transfer_ix = token::TransferChecked {
            from: ctx.accounts.hybrid_nft_account.to_account_info(),
            mint: ctx.accounts.nft_mint.to_account_info(),
            to: ctx.accounts.user_nft_account.to_account_info(),
            authority: hybrid_pool.to_account_info(),
        };

        let cpi_ctx: CpiContext<TransferChecked> = CpiContext::new_with_signer(
            ctx.accounts.nft_program.to_account_info(),
            nft_transfer_ix,
            outer.as_slice(),
        );

        token::transfer_checked(cpi_ctx, 1, ctx.accounts.nft_mint.decimals)?;
        user_data.nft_count -= 1;

        if user_data.token_amount >= hybrid_pool.nft_price && pool_balance >= hybrid_pool.nft_price
        {
            let transfer_ix = TransferChecked {
                from: ctx.accounts.hybrid_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: hybrid_pool.to_account_info(),
            };

            let token_cpi_ctx: CpiContext<TransferChecked> = CpiContext::new_with_signer(
                ctx.accounts.spl_program.to_account_info(),
                transfer_ix,
                outer.as_slice(),
            );
            token::transfer_checked(
                token_cpi_ctx,
                hybrid_pool.nft_price,
                ctx.accounts.token_mint.decimals,
            )?;
            user_data.token_amount -= hybrid_pool.nft_price;
        }
    } else if user_data.token_amount > 0 {
        let transfer_ix = TransferChecked {
            from: ctx.accounts.hybrid_token_account.to_account_info(),
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: hybrid_pool.to_account_info(),
        };

        let token_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.spl_program.to_account_info(),
            transfer_ix,
            outer.as_slice(),
        );

        token::transfer_checked(
            token_cpi_ctx,
            user_data.token_amount,
            ctx.accounts.token_mint.decimals,
        )?;
        user_data.token_amount = 0;
    } else {
        return Err(ErrorCode::EmptyShare.into());
    }

    Ok(())
}
