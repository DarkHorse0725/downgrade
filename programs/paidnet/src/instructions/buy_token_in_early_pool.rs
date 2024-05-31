use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };
use paid_stake::states::Staker;
use crate::pool_logic::{ calculate_participiant_fee, max_purchase_amount_for_early_access, EAELRY_POOL_PARTICIPANT_STAKE_AMOUNT };
use crate::{ PoolStorage, VestingStorage, UserPurchaseAccount, UserVestingAccount };
use crate::error::*;

#[derive(Accounts)]
pub struct BuyTokenInEarlyPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub ido_mint: Account<'info, Mint>,
    pub purchase_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_storage_account: Account<'info, PoolStorage>,
    pub vesting_storage_account: Account<'info, VestingStorage>,

    #[account(mut)]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [b"purchase-vault", pool_storage_account.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = purchase_mint,
        token::authority = purchase_vault
    )]
    pub purchase_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_pot: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staker_account: Account<'info, Staker>,

    #[account(mut)]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(mut)]
    pub user_vesting: Account<'info, UserVestingAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyTokenInEarlyPool<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_purchase_token.to_account_info(),
            to: self.purchase_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
    fn transfer_fee_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_purchase_token.to_account_info(),
            to: self.reward_pot.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

// invest in early pool
pub fn buy_token_in_early_pool_handler(
    ctx: Context<BuyTokenInEarlyPool>,
    purchase_amount: u64,
    purchase_bump: u8
) -> Result<()> {
    let pool_storage: &Account<PoolStorage> = &ctx.accounts.pool_storage_account;
    // validate stake amount
    if ctx.accounts.staker_account.total_staked < EAELRY_POOL_PARTICIPANT_STAKE_AMOUNT {
        return err!(ErrCode::NotEnoughStaker);
    }
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
    let early_purchased: u64 = ctx.accounts.user_purchase_account.early_purchased;

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
    let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
    if !vesting_storage.funded {
        return err!(ErrCode::NotFunded);
    }

    // send token to purchase vault
    token::transfer(ctx.accounts.transfer_ctx(), purchase_amount - participant_fee)?;
    // send fee to stake program 
    token::transfer(ctx.accounts.transfer_fee_ctx(), participant_fee)?;

    // update pool info
    let pool: &mut Account<PoolStorage> = &mut ctx.accounts.pool_storage_account;
    pool.purchase_bump = purchase_bump;
    pool.purchased_amount += purchase_amount;
    // update user vesting info
    let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
    user_vesting.total_amount += ido_amount;
    // update user purchase info
    let user: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;
    user.early_purchased += purchase_amount - participant_fee;
    user.principal += purchase_amount - participant_fee;
    user.fee += participant_fee;

    msg!("Bought token");
    Ok(())
}
