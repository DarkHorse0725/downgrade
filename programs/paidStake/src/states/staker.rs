use anchor_lang::prelude::*;


#[account]
pub struct Staker {
  pub total_staked: u64,
  pub withdraw: u64,
  pub last_update: i64,
}