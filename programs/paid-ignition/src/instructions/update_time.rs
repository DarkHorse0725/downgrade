use anchor_lang::prelude::*;

use crate::{ error::ErrCode, Pool };

#[derive(Accounts)]
pub struct UpdateTime<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
      mut,
      has_one = owner,
    )]
    pub pool: Box<Account<'info, Pool>>,
}

// update market time by creator
pub fn update_time_handler(
    ctx: Context<UpdateTime>,
    early_pool_close_time: i64,
    open_pool_close_time: i64
) -> Result<()> {
    let pool: &mut Box<Account<Pool>> = &mut ctx.accounts.pool;
    if pool.early_pool_open_time > early_pool_close_time {
        return err!(ErrCode::InvalidTime);
    }

    if early_pool_close_time > open_pool_close_time {
        return err!(ErrCode::InvalidTime);
    }

    if open_pool_close_time > pool.tge_date {
        return err!(ErrCode::InvalidTime);
    }
    // update time
    pool.early_pool_close_time = early_pool_close_time;
    pool.open_pool_open_time = early_pool_close_time;
    pool.open_pool_close_time = open_pool_close_time;
    msg!("Updated times");
    Ok(())
}
