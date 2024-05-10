use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::mem::size_of;

use crate::states::Pool;


#[derive(Accounts)]
pub struct InitPool<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,

  pub reward_mint: Account<'info, Mint>,
  pub farm_mint: Account<'info, Mint>,

  #[account(
    init,
    payer = owner,
    space = size_of::<Pool>() + 8,
  )]
  pub farm: Account<'info, Pool>,

  pub system_program: Program<'info, System>,
}

impl <'info>InitPool<'info> {
  pub fn init_pool(
    &mut self,
    reward_decimals: u8,
    farm_decimals: u8,
    reward_per_block: u64,
  ) -> Result<()> {
    self.farm.owner = self.owner.key();
    self.farm.reward_mint = self.reward_mint.key();
    self.farm.reward_per_block = reward_per_block;
    self.farm.farm_mint = self.farm_mint.key();
    self.farm.total_staked = 0;
    self.farm.farmer_count = 0;
    self.farm.reward_decimals = reward_decimals;
    self.farm.farm_decimals = farm_decimals;

    msg!("Create farm");
    Ok(())
  }
}