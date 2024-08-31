// use crate::{
//     _main::main_state::{MainState, ProgramState},
//     constants::SEED_PROGRAM_STATE,
//     error::MyError,
//     utils::{tranfer_nft, verify_and_get_nft_id},
// };
// use anchor_lang::prelude::*;
// use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
// use mpl_token_metadata::{
//     instruction::{builders::{Transfer as MplTransfer, Burn}, InstructionBuilder, TransferArgs},
//     state::{EDITION, PREFIX as METADATA, TOKEN_RECORD_SEED},
//     ID as MPL_ID,
// };
// use solana_program::program::{invoke, invoke_signed};

// pub fn upgrade_nft(ctx: Context<AUpgradeNft>) -> Result<()> {
//     {
//         ctx.accounts.receive_new_nft(ctx.program_id)?;
//     }
//     Ok(())
// }

// #[derive(Accounts)]
// pub struct AUpgradeNft<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,

//     #[account(
//         mut,
//         address = program_state.main_state_id @ MyError::MissMatchMainStateId,
//     )]
//     pub main_state_account: AccountLoader<'info, MainState>,

//     #[account(
//         mut,
//         seeds = [SEED_PROGRAM_STATE],
//         bump,
//     )]
//     pub program_state: Box<Account<'info, ProgramState>>,

//     #[account(mut)]
//     pub new_nft: AccountInfo<'info>,
//     #[account(
//         mut,
//         token::mint = new_nft,
//         token::authority = user,
//     )]
//     pub user_new_nft_ata: Box<Account<'info, TokenAccount>>,
//     #[account(
//         mut,
//         token::mint = new_nft,
//         token::authority = program_state,
//         constraint = program_state_new_nft_ata.amount == 1 @ MyError::NftNotFound
//     )]
//     pub program_state_new_nft_ata: Box<Account<'info, TokenAccount>>,

//     ///CHECK:
//     #[account(
//         mut,
//         seeds=[
//             METADATA.as_ref(),
//             MPL_ID.as_ref(),
//             new_nft.key().as_ref(),
//         ],
//         bump,
//         seeds::program = MPL_ID
//     )]
//     pub new_nft_metadata_account: AccountInfo<'info>,

//     ///CHECK:
//     #[account(
//         mut,
//         seeds=[
//             METADATA.as_ref(),
//             MPL_ID.as_ref(),
//             new_nft.key().as_ref(),
//             EDITION.as_ref(),
//         ],
//         bump,
//         seeds::program = MPL_ID
//     )]
//     pub new_nft_edition_account: AccountInfo<'info>,
//     pub token_program: Program<'info, Token>,

//     // pNFT specific
//     ///CHECK:
//     #[account(
//         mut,
//         seeds = [
//             METADATA.as_ref(),
//             MPL_ID.as_ref(),
//             new_nft.key().as_ref(),
//             TOKEN_RECORD_SEED.as_ref(),
//             user_new_nft_ata.key().as_ref(),
//         ],
//         bump,
//         seeds::program = MPL_ID
//     )]
//     pub user_token_record_account: AccountInfo<'info>,

//     ///CHECK:
//     #[account(
//         mut,
//         seeds = [
//             METADATA.as_ref(),
//             MPL_ID.as_ref(),
//             new_nft.key().as_ref(),
//             TOKEN_RECORD_SEED.as_ref(),
//             program_state_new_nft_ata.key().as_ref(),
//         ],
//         bump,
//         seeds::program = MPL_ID
//     )]
//     pub program_state_token_record_account: AccountInfo<'info>,

//     ///CHECK:
//     #[account(
//         //TODO: we can add extra check
//     )]
//     pub authorization_rules_program: AccountInfo<'info>,
//     ///CHECK:
//     #[account(mut)]
//     pub authorization_rules: AccountInfo<'info>,

//     ///CHECK:
//     #[account()]
//     pub sysvar_instructions: AccountInfo<'info>,

//     ///CHECK:
//     #[account(address = MPL_ID)]
//     pub mpl_program: AccountInfo<'info>,
//     pub ata_program: Program<'info, AssociatedToken>,
//     pub system_program: Program<'info, System>,
// }

// impl<'info> AUpgradeNft<'info> {
//     pub fn receive_new_nft(&mut self, program_id: &Pubkey) -> Result<()> {
//         let sender = self.program_state.to_account_info();
//         let sender_ata = self.program_state_new_nft_ata.to_account_info();
//         let sender_token_record = self.program_state_token_record_account.to_account_info();
//         let nft = self.new_nft.to_account_info();
//         let receiver = self.user.to_account_info();
//         let receiver_ata = self.user_new_nft_ata.to_account_info();
//         let receiver_token_record = self.user_token_record_account.to_account_info();
//         let payer = self.user.to_account_info();
//         let authorization_rules_program = self.authorization_rules_program.to_account_info();
//         let authorization_rules = self.authorization_rules.to_account_info();
//         let metadata = self.new_nft_metadata_account.to_account_info();
//         let edition = self.new_nft_edition_account.to_account_info();
//         let system_program = self.system_program.to_account_info();
//         let sysvar_instructions = self.sysvar_instructions.to_account_info();
//         let token_program = self.token_program.to_account_info();
//         let ata_program = self.ata_program.to_account_info();
//         let mpl_program = self.mpl_program.to_account_info();

//         let ix = MplTransfer {
//             sysvar_instructions: sysvar_instructions.key(),
//             payer: payer.key(),
//             token: sender_ata.key(),
//             system_program: system_program.key(),
//             authorization_rules_program: Some(authorization_rules_program.key()),
//             authorization_rules: Some(authorization_rules.key()),
//             metadata: metadata.key(),
//             token_owner: sender.key(),
//             destination_owner: receiver.key(),
//             destination: receiver_ata.key(),
//             mint: nft.key(),
//             authority: sender.key(),
//             edition: Some(edition.key()),
//             destination_token_record: Some(receiver_token_record.key()),
//             owner_token_record: Some(sender_token_record.key()),
//             spl_ata_program: ata_program.key(),
//             spl_token_program: token_program.key(),
//             args: TransferArgs::V1 {
//                 amount: 1,
//                 authorization_data: None,
//             },
//         }
//         .instruction();

//         let (_, bump) = Pubkey::find_program_address(&[SEED_PROGRAM_STATE], program_id);
//         let res = invoke_signed(
//             &ix,
//             &[
//                 sender,
//                 sender_ata,
//                 sender_token_record,
//                 nft,
//                 receiver,
//                 receiver_ata,
//                 receiver_token_record,
//                 payer,
//                 authorization_rules_program,
//                 authorization_rules,
//                 metadata,
//                 edition,
//                 system_program,
//                 sysvar_instructions,
//                 token_program,
//                 ata_program,
//                 mpl_program,
//             ],
//             &[&[SEED_PROGRAM_STATE, &[bump]]],
//         )?;

//         Ok(())
//     }
// }
