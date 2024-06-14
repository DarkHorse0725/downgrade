use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::{
    calculate_participiant_fee,
    error::ErrCode,
    max_purchase_amount_for_early_access,
    Buyer,
    Pool,
};
use std::mem::size_of;

#[derive(Accounts)]
pub struct BuyInEarlyPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub purchase_mint: Box<Account<'info, Mint>>,

    #[account(
    mut,
    token::mint = purchase_mint,
  )]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"purchase-vault", pool.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = purchase_mint,
        token::authority = purchase_vault
    )]
    pub purchase_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<Buyer>() + 8,
        seeds = [b"buyer", pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub buyer: Box<Account<'info, Buyer>>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyInEarlyPool<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_purchase_token.to_account_info(),
            to: self.purchase_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

pub fn buy_in_early_pool_handler(
    ctx: Context<BuyInEarlyPool>,
    purchase_amount: u64,
    bump: u8
) -> Result<()> {
    let pool_storage: &Box<Account<Pool>> = &ctx.accounts.pool;
    // validate stake amount
    // if ctx.accounts.buyer.total_staked < EAELRY_POOL_PARTICIPANT_STAKE_AMOUNT {
    //     return err!(ErrCode::NotEnoughStaker);
    // }
    // validate time
    let now: i64 = ctx.accounts.clock.unix_timestamp as i64;
    if now > pool_storage.early_pool_close_time {
        return err!(ErrCode::TimeOutBuyIDOToken);
    }
    if now < pool_storage.early_pool_open_time {
        return err!(ErrCode::TimeOutBuyIDOToken);
    }
    // validate amount
    if purchase_amount == 0 {
        return err!(ErrCode::InvalidAmount);
    }
    // calculate purchaseable amounts
    let early_purchased: u64 = ctx.accounts.buyer.early_purchased;

    let allow_purchase_amount: u64 = max_purchase_amount_for_early_access(
        pool_storage.total_raise_amount,
        pool_storage.open_pool_proportion as u64,
        pool_storage.early_pool_proportion as u64
    );

    if early_purchased + purchase_amount > allow_purchase_amount {
        return err!(ErrCode::ExceedMaxPurchaseAmountForEarlyAccess);
    }

    // calculate fee amount
    let participant_fee: u64 = calculate_participiant_fee(
        purchase_amount,
        pool_storage.early_pool_participation_fee_percentage
    );
    let ido_amount: u64 = (purchase_amount - participant_fee) * pool_storage.offered_currency.rate;
    let vesting_storage = &ctx.accounts.pool;
    if !vesting_storage.funded {
        return err!(ErrCode::NotFunded);
    }

    // send token to purchase vault
    token::transfer(ctx.accounts.transfer_ctx(), purchase_amount - participant_fee)?;
    // send fee to stake program
    // token::transfer(ctx.accounts.transfer_fee_ctx(), participant_fee)?;

    // update pool info
    let pool = &mut ctx.accounts.pool;
    pool.purchase_bump = bump;
    pool.purchased_amount += purchase_amount;
    // update user vesting info
    let buyer: &mut Box<Account<Buyer>> = &mut ctx.accounts.buyer;
    buyer.total_amount += ido_amount;
    // update user purchase info
    buyer.early_purchased += purchase_amount - participant_fee;
    buyer.total_purchase += purchase_amount - participant_fee;

    msg!("Bought token");
    Ok(())
}
