use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::mem::size_of;

use crate::states::Pool;

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // @dev mint address of reward token
    pub reward_mint: Box<Account<'info, Mint>>,
    // @dev mint address of stake token
    pub stake_mint: Box<Account<'info, Mint>>,

    // @dev pool account
    #[account(init, payer = owner, space = size_of::<Pool>() + 8)]
    pub pool: Box<Account<'info, Pool>>,

    pub system_program: Program<'info, System>,
}

// @dev initialize pool by owner after deploy
pub fn init_pool_handler(
    ctx: Context<InitPool>,
    reward_decimals: u8,
    farm_decimals: u8,
    reward_per_block: u64
) -> Result<()> {
    // format pool info
    let pool: &mut Account<Pool> = &mut ctx.accounts.pool;
    pool.owner = ctx.accounts.owner.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.reward_per_block = reward_per_block;
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.total_staked = 0;
    pool.staker_count = 0;
    pool.reward_decimals = reward_decimals;
    pool.stake_decimals = farm_decimals;

    msg!("Create pool");
    Ok(())
}
