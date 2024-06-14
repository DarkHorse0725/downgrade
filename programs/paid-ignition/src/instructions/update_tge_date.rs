use anchor_lang::prelude::*;

use crate::{error::ErrCode, Pool, MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS};

#[derive(Accounts)]
pub struct UpdateTgeDate <'info> {
  #[account(mut)]
  pub owner: Signer<'info>,

  // @dev pool account
  #[account(
    mut,
    has_one = owner
  )]
  pub pool: Box<Account<'info, Pool>>,
}

// @dev it allows to update tge date by creator
pub fn update_tge_date_handler(ctx: Context<UpdateTgeDate>, tge_date: i64) -> Result<()> {
  let pool = &mut ctx.accounts.pool;

  // validate new tge date
  if pool.open_pool_close_time > tge_date {
      return err!(ErrCode::InvalidTime);
  }

  if pool.tge_update_attempts >= MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS {
      return err!(ErrCode::NotAllowedToAdjustTGEDateExceedsAttempts);
  }

  pool.tge_date = tge_date;
  msg!("Updated tge date");
  Ok(())
}