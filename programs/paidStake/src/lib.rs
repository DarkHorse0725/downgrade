use anchor_lang::prelude::*;
use states::*;
use error::*;
use instructions::*;

pub mod states;
mod instructions;
pub mod error;

declare_id!("8FcYYJ38nxLKWD8BN6JYs8b3yFnnXzkrL9Pfx43NNUPj");

#[program]
pub mod paid_stake {
    use super::*;

    pub fn init_pool(
        ctx: Context<InitPool>,
        reward_decimals: u8,
        farm_decimals: u8,
        reward_per_block: u64,
    ) -> Result<()> {
        init_pool_handler(ctx, reward_decimals, farm_decimals, reward_per_block)
    }

    pub fn init_reward(
        ctx: Context<FundReward>,
        amount: u64,
        pot_bump: u8,
    ) -> Result<()> {
        fund_reward_handler(ctx, amount, pot_bump)
    }

    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        bump: u8,
    ) -> Result<()> {
       stake_handler(ctx, amount, bump)
    }

    pub fn claim(
        ctx: Context<Claim>,
    ) -> Result<()> {
        claim_handler(ctx)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
    ) -> Result<()> {
        withdraw_handler(ctx, amount)
    }
}
