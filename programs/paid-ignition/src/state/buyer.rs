use anchor_lang::prelude::*;


#[account]
pub struct Buyer {
  // @dev paid total amount of purchase token
  pub total_purchase: u64,
  // @dev paid total amount in early pool, based on purchase token
  pub early_purchased: u64,
  // @dev locked total amount of ido token
  pub total_amount: u64,
  // @dev claimed amount of ido token
  pub cliamed_amount: u64,
}