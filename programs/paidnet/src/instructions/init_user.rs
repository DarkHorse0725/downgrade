use anchor_lang::prelude::*;
use std::mem::size_of;
use crate::{UserPurchaseAccount, UserVestingAccount, PoolStorage};

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(
        init,
        payer = signer,
        space = size_of::<UserPurchaseAccount>() + 8,
        seeds = [b"user-purchase", pool_storage_account.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(
        init,
        payer = signer,
        space = size_of::<UserVestingAccount>() + 8,
        seeds = [b"user-vesting", pool_storage_account.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_vesting: Account<'info, UserVestingAccount>,

    pub system_program: Program<'info, System>,
}


pub fn init_user_handler(ctx: Context<InitUser>) -> Result<()> {
  let user_purchase: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;
  user_purchase.principal = 0;
  user_purchase.fee = 0;
  user_purchase.withdrawn = 0;
  user_purchase.early_purchased = 0;
  let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
  user_vesting.total_amount = 0;
  user_vesting.claimed_amount = 0;
  msg!("Initialized user");
  Ok(())
}