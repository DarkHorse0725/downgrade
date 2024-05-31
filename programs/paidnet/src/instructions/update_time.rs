use anchor_lang::prelude::*;
use crate::{ PoolStorage, VestingStorage };
use crate::error::ErrCode;

#[derive(Accounts)]
pub struct UpdateTime<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner
    )]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(has_one = owner)]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub system_program: Program<'info, System>,
}

// update market time by creator
pub fn update_time_handler(
    ctx: Context<UpdateTime>,
    early_pool_close_time: i64,
    open_pool_close_time: i64
) -> Result<()> {
    let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
    let pool_storage: &mut Account<'_, PoolStorage> = &mut ctx.accounts.pool_storage_account;
    if pool_storage.early_pool_open_time > early_pool_close_time {
        return err!(ErrCode::InvalidTime);
    }

    if early_pool_close_time > open_pool_close_time {
        return err!(ErrCode::InvalidTime);
    }

    if open_pool_close_time > vesting_storage.tge_date {
        return err!(ErrCode::InvalidTime);
    }
    // update time
    pool_storage.early_pool_close_time = early_pool_close_time;
    pool_storage.open_pool_open_time = early_pool_close_time;
    pool_storage.open_pool_close_time = open_pool_close_time;
    msg!("Updated times");
    Ok(())
}
