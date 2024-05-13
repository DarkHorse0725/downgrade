use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token::{ Mint, Token } };
use crate::{pool_logic::PERCENTAGE_DENOMINATOR, state::{ PoolStorage, VestingStorage }};
use std::mem::size_of;
use crate::error::ErrCode;

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub purchase_mint: Account<'info, Mint>,
    pub ido_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<PoolStorage>() + 8,
    )]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<VestingStorage>() + 8,
        seeds = [
            b"vesting_storage",
            pool_storage_account.key().as_ref()
        ],
        bump
    )]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_pool_handler(
    ctx: Context<CreatePool>,
    uints: [u64; 18],
) -> Result<()> {
    // validate inputs
    if uints[2] > PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidTokenFeePercentage);
    }

    if uints[5] == 0 {
        return err!(ErrCode::InvalidAmount);
    }

    if uints[5] >= PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidGalaxyPoolProportion);
    }

    if uints[6] >= PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidEarlyAccessProportion);
    }

    if uints[8] + uints[9] + uints[10] > uints[13] {
        return err!(ErrCode::InvalidTime);
    }

    if uints[7] == 0 {
        return err!(ErrCode::InvalidAmount);
    }

    if uints[14] >= PERCENTAGE_DENOMINATOR {
        return err!(ErrCode::InvalidTGEPercentage);
    }

    // create pool
    let pool_storage: &mut Account<'_, PoolStorage> = &mut ctx.accounts.pool_storage_account;
    pool_storage.owner = ctx.accounts.signer.key();
    if uints[0] <= uints[1] {
        return err!(ErrCode::MaxPurchaseForKYCUserNotValid);
    }
    pool_storage.max_purchase_amount_for_kyc_user = uints[0];
    pool_storage.max_purchase_amount_for_not_kyc_user = uints[1];

    pool_storage.token_fee_percentage = uints[2] as u16;
    pool_storage.early_pool_participation_fee_percentage = uints[3] as u16;
    pool_storage.open_pool_participation_fee_percentage = uints[4] as u16;
    pool_storage.open_pool_proportion = uints[5] as u16;
    pool_storage.early_pool_proportion = uints[6] as u16;
    pool_storage.total_raise_amount = uints[7];
    pool_storage.early_pool_open_time = uints[8] as i64;
    pool_storage.early_pool_close_time = (uints[8] + uints[9]) as i64;
    pool_storage.open_pool_open_time = (uints[8] + uints[9]) as i64;
    pool_storage.open_pool_close_time = pool_storage.open_pool_open_time + (uints[10] as i64);
    pool_storage.offered_currency.rate = uints[11];
    pool_storage.offered_currency.decimals = uints[12] as u16;
    pool_storage.purchase_token = ctx.accounts.purchase_mint.key();

    // create  vesting
    let vesting_storage: &mut Account<VestingStorage> = &mut ctx.accounts.vesting_storage_account;
    vesting_storage.ido_token = ctx.accounts.ido_mint.key();
    vesting_storage.tge_date = uints[13] as i64;
    vesting_storage.tge_percentage = uints[14] as u16;
    vesting_storage.vesting_cliff = uints[15] as i64;
    vesting_storage.vesting_freguency = uints[16];
    vesting_storage.number_of_vesting_release = uints[17];
    vesting_storage.owner = ctx.accounts.signer.key();
    vesting_storage.claimable = true;
    vesting_storage.initial_tge_date = uints[13] as i64;
    msg!("Pool created");
    Ok(())
}
