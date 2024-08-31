use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("token address is incorrect.")]
    IncorrectTokenAddress,
    #[msg("mint address is incorrect.")]
    IncorrectMintAddress,
    #[msg("owner address is incorrect.")]
    IncorrectOwnerAddress,
    #[msg("proof is incorrect.")]
    InvalidProof,
    #[msg("you have no share")]
    EmptyShare,
    #[msg("Bad Metadata")]
    BadMetadata,
    #[msg("Bad Ruleset")]
    BadRuleset,
}
