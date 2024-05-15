use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::mem::size_of;

use crate::states::Pool;


#[derive(Accounts)]
pub struct InitPool<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,

  pub reward_mint: Account<'info, Mint>,
  pub stake_mint: Account<'info, Mint>,

  #[account(
    init,
    payer = owner,
    space = size_of::<Pool>() + 8,
  )]
  pub pool: Account<'info, Pool>,

  pub system_program: Program<'info, System>,
}


pub fn init_pool_handler(
  ctx: Context<InitPool>, 
  reward_decimals: u8,
  farm_decimals: u8,
  reward_per_block: u64,
) -> Result<()> {
  let pool: &mut Account<Pool> = &mut ctx.accounts.pool;
  pool.owner = ctx.accounts.owner.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.reward_per_block = reward_per_block;
    pool.farm_mint = ctx.accounts.stake_mint.key();
    pool.total_staked = 0;
    pool.staker_count = 0;
    pool.reward_decimals = reward_decimals;
    pool.farm_decimals = farm_decimals;

    msg!("Create farm");
    Ok(())
}