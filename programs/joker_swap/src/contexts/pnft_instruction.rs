use anchor_lang::prelude::*;

use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint as TokenMint, Token, TokenAccount as AccountToken},
};
use mpl_token_auth_rules::payload::{Payload, PayloadType, ProofInfo, SeedsVec};

use mpl_token_metadata::types::{AuthorizationData, Payload as AuthorizationPayload};
#[derive(Accounts)]
pub struct ProgNftShared<'info> {
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: address below
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// CHECK: address below
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    /// CHECK: address below
    #[account(address = mpl_token_auth_rules::ID)]
    pub authorization_rules_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct TransferPNFT<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    pub receiver: AccountInfo<'info>,
    #[account(mut)]
    pub src: Box<Account<'info, AccountToken>>,
    #[account(mut)]
    pub dest: Box<Account<'info, AccountToken>>,
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
                "edition".as_bytes(),
                src.key().as_ref()
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
                "edition".as_bytes(),
                dest.key().as_ref()
            ],
            seeds::program = mpl_token_metadata::ID,
            bump
        )]
    pub dest_token_record: UncheckedAccount<'info>,
    pub pnft_shared: ProgNftShared<'info>,
    //
    // remaining accounts could be passed, in this order:
    // - rules account
    // - mint_whitelist_proof
    // - creator_whitelist_proof
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct AuthorizationDataLocal {
    pub payload: Vec<TaggedPayload>,
}
impl From<AuthorizationDataLocal> for AuthorizationData {
    fn from(val: AuthorizationDataLocal) -> Self {
        let mut p = Payload::new();
        val.payload.into_iter().for_each(|tp| {
            p.insert(tp.name, PayloadType::try_from(tp.payload).unwrap());
        });
        let payload =
            AuthorizationPayload::try_from_slice(p.try_to_vec().unwrap().as_slice()).unwrap();
        AuthorizationData { payload }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TaggedPayload {
    name: String,
    payload: PayloadTypeLocal,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum PayloadTypeLocal {
    /// A plain `Pubkey`.
    Pubkey(Pubkey),
    /// PDA derivation seeds.
    Seeds(SeedsVecLocal),
    /// A merkle proof.
    MerkleProof(ProofInfoLocal),
    /// A plain `u64` used for `Amount`.
    Number(u64),
}
impl From<PayloadTypeLocal> for PayloadType {
    fn from(val: PayloadTypeLocal) -> Self {
        match val {
            PayloadTypeLocal::Pubkey(pubkey) => PayloadType::Pubkey(pubkey),
            PayloadTypeLocal::Seeds(seeds) => {
                PayloadType::Seeds(SeedsVec::try_from(seeds).unwrap())
            }
            PayloadTypeLocal::MerkleProof(proof) => {
                PayloadType::MerkleProof(ProofInfo::try_from(proof).unwrap())
            }
            PayloadTypeLocal::Number(number) => PayloadType::Number(number),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SeedsVecLocal {
    /// The vector of derivation seeds.
    pub seeds: Vec<Vec<u8>>,
}
impl From<SeedsVecLocal> for SeedsVec {
    fn from(val: SeedsVecLocal) -> Self {
        SeedsVec { seeds: val.seeds }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ProofInfoLocal {
    /// The merkle proof.
    pub proof: Vec<[u8; 32]>,
}
impl From<ProofInfoLocal> for ProofInfo {
    fn from(val: ProofInfoLocal) -> Self {
        ProofInfo { proof: val.proof }
    }
}
