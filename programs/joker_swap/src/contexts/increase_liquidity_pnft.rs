use crate::states::{Config, HybridPool, PnftTokenAccountAddress, UserData};
use crate::utils::verify;
use anchor_lang::solana_program::keccak;
use anchor_spl::token_2022::spl_token_2022::solana_zk_token_sdk::curve25519::scalar::Zeroable;

use crate::pnft::send_pnft;
use crate::ErrorCode;
use anchor_lang::prelude::*;
// use anchor_lang::solana_program::keccak;

// use anchor_spl::{
//     self,
//     token_interface::{self as token, Mint, TokenAccount, TokenInterface, TransferChecked},
// };
use super::pnft_instruction::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint as TokenMint, Token, TokenAccount as AccountToken},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(id:u16)]
pub struct IncreaseLiquidityPNFT<'info> {
    #[account(mut,seeds = [b"config"],bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(init_if_needed,space=64+33, payer = signer, seeds = [b"user-data",signer.key().as_ref() ,pool_account.key().as_ref()], bump)]
    pub user_data: Box<Account<'info, UserData>>,
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
    #[account(mut)]
    pub hybrid_nft_account: Box<Account<'info, AccountToken>>,
    pub nft_mint: Box<Account<'info, TokenMint>>,
    // misc
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(init_if_needed, payer = signer,space=40,seeds=[b"token-pda-account", pool_account.key().as_ref(), hybrid_nft_account.key().as_ref()] ,bump)]
    #[account(constraint =  nft_account_pda_address.token_account == hybrid_nft_account.key() || nft_account_pda_address.token_account == Pubkey::zeroed())]
    pub nft_account_pda_address: Box<Account<'info, PnftTokenAccountAddress>>,
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
            ],
            seeds::program = mpl_token_metadata::ID,
            bump
        )]
    pub pool_token_record: UncheckedAccount<'info>,
    pub spl_program: Interface<'info, TokenInterface>,
    pub pnft_shared: ProgNftShared<'info>,
}

pub fn increase_liquidity_pnft<'info>(
    ctx: Context<'_, '_, '_, 'info, IncreaseLiquidityPNFT<'info>>,
    _id: u16,
    proof: Vec<[u8; 32]>,
    authorization_data: Option<AuthorizationDataLocal>,
    rules_acc_present: bool,
) -> Result<()> {
    let hybrid_pool = &mut ctx.accounts.pool_account;
    let token_account_pda_address = &mut ctx.accounts.nft_account_pda_address;
    token_account_pda_address.token_account = ctx.accounts.hybrid_nft_account.key();
    let user_data: &mut Box<Account<'info, UserData>> = &mut ctx.accounts.user_data;
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

    let rem_acc = &mut ctx.remaining_accounts.iter();

    let auth_rules = if rules_acc_present {
        Some(next_account_info(rem_acc)?)
    } else {
        None
    };
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
        None,
    )?;

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
    user_data.nft_count += 1;
    user_data.token_amount += hybrid_pool.nft_price;
    Ok(())
}
