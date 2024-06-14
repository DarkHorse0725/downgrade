use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ self, Token, TokenAccount, Transfer },
};

use crate::{ Buyer, Pool };

#[derive(Accounts)]
pub struct UserWithdrawPurchase<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
      mut,
      seeds = [b"buyer", pool.key().as_ref(), signer.key().as_ref()],
      bump
    )]
    pub buyer: Box<Account<'info, Buyer>>,

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

// @dev allowed to withdraw purchase token by user if pool was failed
pub fn user_withdraw_purchase_handler(
    ctx: Context<UserWithdrawPurchase>,
    amount: u64
) -> Result<()> {
    let pool_storage: &Box<Account<Pool>> = &ctx.accounts.pool;
    // seed of authority pda of purchase vault
    let seeds: &[&[u8]; 3] = &[
        b"purchase-vault",
        pool_storage.to_account_info().key.as_ref(),
        &[pool_storage.purchase_bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    // transfer token to user token account
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;
    msg!("Withdraw purchase token");
    Ok(())
}
