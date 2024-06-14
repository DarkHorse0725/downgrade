pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7bv1WyCMQMFB61TE76VWUtXZLq3n8wWnS88XNLYFNekd");

#[program]
pub mod paid_ignition {
    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>, uints: [u64; 18]) -> Result<()> {
        create_pool_handler(ctx, uints)
    }

    pub fn fund_offer(ctx: Context<FundOffer>, amount: u64, bump: u8) -> Result<()> {
        fund_offer_handler(ctx, amount, bump)
    }

    pub fn update_tge_date(ctx: Context<UpdateTgeDate>, tge_date: i64) -> Result<()> {
        update_tge_date_handler(ctx, tge_date)
    }

    pub fn update_time(
        ctx: Context<UpdateTime>,
        early_pool_close_time: i64,
        open_pool_close_time: i64
    ) -> Result<()> {
        update_time_handler(ctx, early_pool_close_time, open_pool_close_time)
    }

    pub fn buy_in_early_pool(
        ctx: Context<BuyInEarlyPool>,
        purchase_amount: u64,
        bump: u8
    ) -> Result<()> {
        buy_in_early_pool_handler(ctx, purchase_amount, bump)
    }

    pub fn buy_in_open_pool(
        ctx: Context<BuyInOpenPool>,
        purchase_amount: u64,
        bump: u8
    ) -> Result<()> {
        buy_in_open_pool_handler(ctx, purchase_amount, bump)
    }

    pub fn user_withdraw_purchase(ctx: Context<UserWithdrawPurchase>, amount: u64) -> Result<()> {
        user_withdraw_purchase_handler(ctx, amount)
    }

    pub fn withdraw_offer(ctx: Context<WithdrawOffer>, amount: u64) -> Result<()> {
        withdraw_offer_handler(ctx, amount)
    }

    pub fn claim_offer(ctx: Context<ClaimOffer>) -> Result<()> {
        claim_offer_handler(ctx)
    }
}
