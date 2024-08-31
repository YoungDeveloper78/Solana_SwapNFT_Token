use crate::states::HybridPool;
use anchor_lang::prelude::*;
#[derive(Accounts)]
#[instruction(id:u16,bump:u8)]
pub struct ChangeRoot<'info> {
    #[account(mut,seeds = [b"pool",signer.key().as_ref(),id.to_le_bytes().as_ref()], bump=bump)]
    pub pool_account: Box<Account<'info, HybridPool>>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
pub fn change_root(ctx: Context<ChangeRoot>, _id: u16, _bump: u8, root: [u8; 32]) -> Result<()> {
    let hybrid_pool = &mut ctx.accounts.pool_account;
    hybrid_pool.root_mint = root;
    Ok(())
}
