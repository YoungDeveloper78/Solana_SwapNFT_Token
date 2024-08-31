use crate::states::{Config, HybridPool};
use crate::utils::verify;

use crate::ErrorCode;
use anchor_lang::solana_program::keccak;

use anchor_lang::prelude::*;

use anchor_spl::{
    self,
    token_interface::{self as token, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(id:u16)]

pub struct Swap<'info> {
    #[account(mut,constraint = pool_account.token_mint== token_mint.key()&& pool_account.id == id)]
    pub pool_account: Box<Account<'info, HybridPool>>,
    #[account(mut,seeds = [b"config"],bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, seeds = [b"token-pool", pool_account.key().as_ref(), token_mint.key().as_ref()], bump)]
    pub hybrid_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(init_if_needed,payer=signer, seeds = [b"nft-pool", pool_account.key().as_ref(), nft_mint.key().as_ref()], bump,token::mint = nft_mint, token::authority = pool_account,token::token_program=nft_program)]
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
pub fn nft_to_token(ctx: Context<Swap>, id: u16, bump: u8, proof: Vec<[u8; 32]>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let hybrid_pool: &mut Account<HybridPool> = &mut ctx.accounts.pool_account;
    let amount_out = hybrid_pool.nft_price;
    let nft_mint_keccak = keccak::hash(ctx.accounts.nft_mint.key().as_ref());

    require!(
        verify(proof, hybrid_pool.root_mint, nft_mint_keccak.0),
        ErrorCode::InvalidProof
    );
    let fee = config.swap_fee;

    let ix = anchor_lang::solana_program::system_instruction::transfer(
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

    let bump_vector = bump.to_le_bytes();
    let id_vector = id.to_le_bytes();

    let inner = vec![
        b"pool".as_ref(),
        hybrid_pool.owner.as_ref(),
        &id_vector,
        &bump_vector,
    ];
    let outer: Vec<&[&[u8]]> = vec![inner.as_slice()];

    let token_transfer_ix = TransferChecked {
        from: ctx.accounts.hybrid_token_account.to_account_info(),
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: hybrid_pool.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.spl_program.to_account_info(),
        token_transfer_ix,
        outer.as_slice(),
    );

    token::transfer_checked(cpi_ctx, amount_out, ctx.accounts.token_mint.decimals)?;

    Ok(())
}
pub fn token_to_nft(ctx: Context<Swap>, id: u16, bump: u8, proof: Vec<[u8; 32]>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let hybrid_pool = &mut ctx.accounts.pool_account;
    let nft_mint_keccak = keccak::hash(ctx.accounts.nft_mint.key().as_ref());
    require!(
        verify(proof, hybrid_pool.root_mint, nft_mint_keccak.0),
        ErrorCode::InvalidProof
    );
    let fee = config.swap_fee;

    let ix = anchor_lang::solana_program::system_instruction::transfer(
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

    let bump_vector = bump.to_le_bytes();
    let id_vector = id.to_le_bytes();

    let inner = vec![
        b"pool".as_ref(),
        hybrid_pool.owner.as_ref(),
        &id_vector,
        &bump_vector,
    ];
    let outer = vec![inner.as_slice()];
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

    Ok(())
}
