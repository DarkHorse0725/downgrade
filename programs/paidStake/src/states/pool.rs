use anchor_lang::prelude::*;


#[account]
pub struct Pool {
  // @dev pool owner
  pub owner: Pubkey,
  // @dev reward per block
  pub reward_per_block: u64,
  // @dev mint address of reward token
  pub reward_mint: Pubkey,
  // @dev mint address of stake token
  pub stake_mint: Pubkey,
  // @dev pda bump of reward pot
  pub pot_bump: u8,
  // @dev pda bump of stake vault
  pub vault_bump: u8,
  // @dev total staked amount, based on stake token
  pub total_staked: u64,
  // @dev staker count
  pub staker_count: u64,
  // @dev reward token decimals 
  pub reward_decimals:u8,
  // @dev stake token decimals
  pub stake_decimals: u8
}