use anchor_lang::prelude::*;


#[account]
pub struct Staker {
  // @dev total staked amount
  pub total_staked: u64,
  // @dev claimed amount
  pub withdraw: u64,
  // @dev last claim or deposit time
  pub last_update: i64,
}