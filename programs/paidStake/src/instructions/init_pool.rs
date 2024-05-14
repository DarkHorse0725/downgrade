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

impl <'info>InitPool<'info> {
  pub fn init_pool(
    &mut self,
    reward_decimals: u8,
    farm_decimals: u8,
    reward_per_block: u64,
  ) -> Result<()> {
    self.pool.owner = self.owner.key();
    self.pool.reward_mint = self.reward_mint.key();
    self.pool.reward_per_block = reward_per_block;
    self.pool.farm_mint = self.stake_mint.key();
    self.pool.total_staked = 0;
    self.pool.staker_count = 0;
    self.pool.reward_decimals = reward_decimals;
    self.pool.farm_decimals = farm_decimals;

    msg!("Create farm");
    Ok(())
  }
}