pub mod state;
pub mod error;
pub mod vesting_logic;
pub mod pool_logic;
mod instructions;

use anchor_lang::prelude::*;
use crate::state::*;
use instructions::*;


declare_id!("AqwFRLotetpQpfVSF9pPFAR1MqB9NmY3a9fUyjJ9nBCv");

#[program]
pub mod paidnet {
    use super::*;

    // pool functions
    pub fn create_pool(
        ctx: Context<CreatePool>,
        uints: [u64; 18],
    ) -> Result<()> {
        create_pool_handler(ctx, uints)
    }

    pub fn update_time(
        ctx: Context<UpdateTime>,
        early_pool_close_time: i64,
        open_pool_close_time: i64
    ) -> Result<()> {
        update_time_handler(ctx, early_pool_close_time, open_pool_close_time)
    }

    pub fn update_tge_date(ctx: Context<UpdateTGEDate>, tge_date: i64) -> Result<()> {
        update_tge_date_handler(ctx, tge_date)
    }

    pub fn buy_token_in_early_pool(
        ctx: Context<BuyTokenInEarlyPool>,
        purchase_amount: u64,
        purchase_bump: u8
    ) -> Result<()> {
        buy_token_in_early_pool_handler(ctx, purchase_amount, purchase_bump)
    }

    pub fn buy_token_in_open_pool(
        ctx: Context<BuyTokenInOpenPool>,
        purchase_amount: u64,
        index: u32,
        root: [u8; 32],
        note: String,
        user_type: String,
        purchase_bump: u8
    ) -> Result<()> {
        buy_token_in_open_pool_handler(ctx, purchase_amount, index, root, note, user_type, purchase_bump)
    }

    pub fn fund_ido_token(ctx: Context<FundIDO>, amount: u64, bump: u8) -> Result<()> {
        fund_ido_handler(ctx, amount, bump)
    }

    // when failed
    pub fn withdraw_ido_token(ctx: Context<WithdrawIDOToken>, amount: u64) -> Result<()> {
        withdraw_ido_handler(ctx, amount)
    }

    // when failed
    pub fn user_withdraw_purchase(ctx: Context<UserWithdrawPurchase>, amount: u64) -> Result<()> {
        user_withdraw_purchase_handler(ctx, amount)
    }

    // when success
    pub fn unlock_ido(ctx: Context<UnlockIDO>) -> Result<()> {
        unlock_ido_handler(ctx)
    }
}
