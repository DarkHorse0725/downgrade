use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ self, Token, TokenAccount, Transfer },
};

use crate::{ state::{ UserPurchaseAccount, UserVestingAccount }, PoolStorage };

#[derive(Accounts)]
pub struct UserWithdrawPurchase<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_vesting: Account<'info, UserVestingAccount>,

    #[account(mut)]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(mut, constraint = signer.key() == pool_storage_account.owner)]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(mut)]
    pub purchase_vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> UserWithdrawPurchase<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.purchase_vault.to_account_info(),
            to: self.user_purchase_token.to_account_info(),
            authority: self.purchase_vault.to_account_info(),
        })
    }
}

pub fn user_withdraw_purchase_handler(
    ctx: Context<UserWithdrawPurchase>,
    amount: u64
) -> Result<()> {
    let pool_storage: &Account<PoolStorage> = &ctx.accounts.pool_storage_account;
    // send spl-token
    let seeds: &[&[u8]; 3] = &[
        b"purchase-vault",
        pool_storage.to_account_info().key.as_ref(),
        &[pool_storage.purchase_bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;
    msg!("Withdraw purchase token");
    Ok(())
}
