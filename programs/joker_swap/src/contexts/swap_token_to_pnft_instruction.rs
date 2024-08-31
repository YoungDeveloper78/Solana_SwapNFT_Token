use crate::states::{Config, HybridPool};
use crate::utils::verify;
use anchor_lang::solana_program::keccak;

use crate::pnft::send_pnft;
use crate::ErrorCode;
use anchor_lang::prelude::*;

use super::pnft_instruction::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint as TokenMint, Token, TokenAccount as AccountToken},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(id:u16)]
pub struct SwapTokenToPNFT<'info> {
    #[account(mut,seeds = [b"config"],bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut,constraint = pool_account.token_mint== token_mint.key()&& pool_account.id == id)]
    pub pool_account: Box<Account<'info, HybridPool>>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, seeds = [b"token-pool", pool_account.key().as_ref(), token_mint.key().as_ref()], bump)]
    pub hybrid_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_nft_account: Box<Account<'info, AccountToken>>,
    #[account(init_if_needed,payer=signer, seeds = [b"nft-pool", pool_account.key().as_ref(), nft_mint.key().as_ref()], bump,token::mint = nft_mint, token::authority = pool_account)]
    pub hybrid_nft_account: Box<Account<'info, AccountToken>>,
    pub nft_mint: Box<Account<'info, TokenMint>>,
    // misc
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // pfnt
    //can't deserialize directly coz Anchor traits not implemented
    // / CHECK: assert_decode_metadata + seeds below
    #[account(
        mut,
        seeds=[
            "metadata".as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            nft_mint.key().as_ref(),
        ],
        seeds::program = mpl_token_metadata::ID,
        bump
    )]
    /// CHECK: seeds below
    pub nft_metadata: UncheckedAccount<'info>,
    //note that MASTER EDITION and EDITION share the same seeds, and so it's valid to check them here
    /// CHECK: seeds below
    #[account(
        seeds=[
            "metadata".as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            nft_mint.key().as_ref(),
            "edition".as_bytes(),
        ],
        seeds::program = mpl_token_metadata::ID,
        bump
    )]
    pub edition: UncheckedAccount<'info>,
    /// CHECK: seeds below
    #[account(mut,
            seeds=[
                "metadata".as_bytes(),
                mpl_token_metadata::ID.as_ref(),
                nft_mint.key().as_ref(),
                "token_record".as_bytes(),
                user_nft_account.key().as_ref()
                // "metadata".as_bytes(),
                // mpl_token_metadata::ID.as_ref(),
                // nft_mint.key().as_ref(),
                // "edition".as_bytes(),
                // user_nft_account.key().as_ref()
            ],
            seeds::program = mpl_token_metadata::ID,
            bump
        )]
    pub owner_token_record: UncheckedAccount<'info>,
    /// CHECK: seeds below
    #[account(mut,
            seeds=[
                "metadata".as_bytes(),
                mpl_token_metadata::ID.as_ref(),
                nft_mint.key().as_ref(),
                "token_record".as_bytes(),
                hybrid_nft_account.key().as_ref()
                // "metadata".as_bytes(),
                // mpl_token_metadata::ID.as_ref(),
                // nft_mint.key().as_ref(),
                // "edition".as_bytes(),
                // hybrid_nft_account.key().as_ref()
            ],
            seeds::program = mpl_token_metadata::ID,
            bump
        )]
    pub pool_token_record: UncheckedAccount<'info>,
    pub spl_program: Interface<'info, TokenInterface>,
    pub pnft_shared: ProgNftShared<'info>,
}

pub fn token_to_pnft<'info>(
    ctx: Context<'_, '_, '_, 'info, SwapTokenToPNFT<'info>>,

    id: u16,
    bump: u8,
    proof: Vec<[u8; 32]>,
    authorization_data: Option<AuthorizationDataLocal>,
    rules_acc_present: bool,
) -> Result<()> {
    // let config = &mut ctx.accounts.config;
    let hybrid_pool: &mut Account<HybridPool> = &mut ctx.accounts.pool_account;

    let nft_mint_keccak = keccak::hash(ctx.accounts.nft_mint.key().as_ref());
    let rem_acc = &mut ctx.remaining_accounts.iter();

    require!(
        verify(proof, hybrid_pool.root_mint, nft_mint_keccak.0),
        ErrorCode::InvalidProof
    );

    let auth_rules = if rules_acc_present {
        Some(next_account_info(rem_acc)?)
    } else {
        None
    };

    transfer_checked(
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
    send_pnft(
        &ctx.accounts.signer.to_account_info(),
        &ctx.accounts.signer.to_account_info(),
        &ctx.accounts.user_nft_account,
        &ctx.accounts.hybrid_nft_account,
        &hybrid_pool.to_account_info(),
        &ctx.accounts.nft_mint,
        &ctx.accounts.nft_metadata,
        &ctx.accounts.edition,
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.pnft_shared.instructions,
        &ctx.accounts.owner_token_record,
        &ctx.accounts.pool_token_record,
        &ctx.accounts.pnft_shared.authorization_rules_program,
        auth_rules,
        authorization_data,
        Some(&outer),
    )?;
    Ok(())
}
