use anchor_lang::prelude::*;

use crate::pool_logic::{MAX_TGE_DATE_ADJUSTMENT, MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS};
use crate::{ PoolStorage, VestingStorage };
use crate::error::*;

#[derive(Accounts)]
pub struct UpdateTGEDate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(constraint = pool_storage_account.owner == signer.key())]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account( 
        mut,
        constraint = vesting_storage_account.owner == signer.key()
    )]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub system_program: Program<'info, System>,
}

pub fn update_tge_date_handler(ctx: Context<UpdateTGEDate>, tge_date: i64) -> Result<()> {
    let vesting_storage: &mut Account<VestingStorage> = &mut ctx.accounts.vesting_storage_account;
    let pool_storage: &Account<PoolStorage> = &ctx.accounts.pool_storage_account;

    if pool_storage.open_pool_close_time > tge_date {
        return err!(ErrCode::InvalidTime);
    }

    if vesting_storage.tge_update_attempts >= MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS {
        return err!(ErrCode::NotAllowedToAdjustTGEDateExceedsAttempts);
    }

    if tge_date > vesting_storage.initial_tge_date + MAX_TGE_DATE_ADJUSTMENT {
        return err!(ErrCode::NotAllowedToAdjustTGEDateTooFar);
    }

    vesting_storage.tge_date = tge_date;
    msg!("Updated tge date");
    Ok(())
}
