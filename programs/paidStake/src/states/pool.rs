use anchor_lang::prelude::*;


#[account]
pub struct Pool {
  pub owner: Pubkey,
  pub reward_per_block: u64,
  pub reward_mint: Pubkey,
  pub farm_mint: Pubkey,
  pub pot_bump: u8,
  pub vault_bump: u8,
  pub total_staked: u64,
  pub farmer_count: u64,
  pub reward_decimals:u8,
  pub farm_decimals: u8
}