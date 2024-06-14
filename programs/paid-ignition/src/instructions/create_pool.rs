use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token };
use crate::{error::ErrCode, state::Pool, PERCENTAGE_DENOMINATOR};
use std::mem::size_of;

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub purchase_mint: Box<Account<'info, Mint>>,
    pub offer_mint: Box<Account<'info, Mint>>,

    #[account(init, payer = creator, space = size_of::<Pool>() + 8)]
    pub pool: Box<Account<'info, Pool>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// create launchpad
pub fn create_pool_handler(ctx: Context<CreatePool>, uints: [u64; 18]) -> Result<()> {
    // validate inputs
    // token fee percentage
    if uints[2] > PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidTokenFeePercentage);
    }
    // early pool proportion
    if uints[5] > PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidTokenFeePercentage);
    }
    // open pool proportion
    if uints[6] > PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidTokenFeePercentage);
    }
    if uints[8] > uints[9] {
        return err!(ErrCode::InvalidTime);
    }
    if uints[9] > uints[10] {
        return err!(ErrCode::InvalidTime);
    }
    if uints[10] > uints[13] {
        return err!(ErrCode::InvalidTime);
    }
    // format pool info
    let pool: &mut Box<Account<Pool>> = &mut ctx.accounts.pool;
    pool.max_purchase_amount_for_kyc_user = uints[0];
    pool.max_purchase_amount_for_not_kyc_user = uints[1];
    pool.token_fee_percentage = uints[2] as u16;
    pool.early_pool_participation_fee_percentage = uints[3] as u16;
    pool.open_pool_participation_fee_percentage = uints[4] as u16;
    pool.early_pool_proportion = uints[5] as u16;
    pool.open_pool_proportion = uints[6] as u16;
    pool.total_raise_amount = uints[7];
    pool.early_pool_open_time = uints[8] as i64;
    pool.early_pool_close_time = uints[9] as i64;
    pool.open_pool_open_time = uints[9] as i64;
    pool.open_pool_close_time = uints[10] as i64;

    pool.offered_currency.rate = uints[11];
    pool.offered_currency.decimals = uints[12] as u8;
    pool.tge_date = uints[13] as i64;
    pool.tge_percentage = uints[14] as u16;
    pool.vesting_cliff = uints[15] as i64;
    pool.vesting_frequency = uints[16] as i64;
    pool.number_of_vesting = uints[17] as i64;
    pool.owner = ctx.accounts.creator.key();
    pool.total_funded_amount = 0;
    pool.offered_currency.mint = ctx.accounts.offer_mint.key();
    pool.purchase_currency.mint = ctx.accounts.purchase_mint.key();
    msg!("Pool created");
    Ok(())
}
