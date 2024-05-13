pub mod state;
pub mod error;
pub mod vesting_logic;
pub mod pool_logic;
mod instructions;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{ self, Transfer, TokenAccount, Mint, Token };
use state::{ UserPurchaseAccount, UserVestingAccount };
use crate::error::ErrCode;
use crate::state::*;
use anchor_spl::associated_token::{ self, Create };
use crate::vesting_logic::calculate_claimable_amount;
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
        let pool_storage: &mut Account<PoolStorage> = &mut ctx.accounts.pool_storage_account;

        // send spl-token
        let seeds: &[&[u8]; 2] = &[
            pool_storage.to_account_info().key.as_ref(),
            &[pool_storage.purchase_bump],
        ];
        let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.purchase_vault.to_account_info(),
            to: ctx.accounts.user_purchase_token.to_account_info(),
            authority: ctx.accounts.purchase_vault.to_account_info(),
        };
        let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts).with_signer(
            signer
        );
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    // when success
    pub fn unlock_ido(ctx: Context<UnlockIDO>) -> Result<()> {
        let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
        let user_purchase: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;
        let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
        if !vesting_storage.claimable {
            return err!(ErrCode::NotClaimable);
        }

        if user_purchase.withdrawn >= user_vesting.total_amount {
            return err!(ErrCode::AlreadyClaimedTotoalAmount);
        }

        if ctx.accounts.user_token.data_is_empty() {
            let cpi_accounts: Create = Create {
                payer: ctx.accounts.signer.to_account_info(),
                associated_token: ctx.accounts.user_token.clone(),
                authority: ctx.accounts.signer.to_account_info(),
                mint: ctx.accounts.ido_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_program: AccountInfo = ctx.accounts.associated_token_program.to_account_info();
            let cpi_ctx: CpiContext<Create> = CpiContext::new(cpi_program, cpi_accounts);
            associated_token::create(cpi_ctx)?;
        }
        // check vesting amount
        let clock: Clock = Clock::get()?;
        let claimable_amount: u64 = calculate_claimable_amount(
            user_vesting.total_amount,
            user_purchase.withdrawn,
            vesting_storage.tge_percentage,
            vesting_storage.tge_date,
            vesting_storage.vesting_cliff,
            vesting_storage.vesting_freguency,
            vesting_storage.number_of_vesting_release,
            clock.unix_timestamp
        );

        if claimable_amount == 0 {
            return err!(ErrCode::NotclaimableAmount);
        }

        // send ido token to user
        let vesting_storage: &mut Account<
            '_,
            VestingStorage
        > = &mut ctx.accounts.vesting_storage_account;
        let seeds: &[&[u8]; 2] = &[
            vesting_storage.to_account_info().key.as_ref(),
            &[vesting_storage.vault_bump],
        ];
        let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts).with_signer(
            signer
        );
        token::transfer(cpi_ctx, claimable_amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UserWithdrawPurchase<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_vesting: Account<'info, UserVestingAccount>,

    #[account(mut)]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(mut, constraint = signer.key() == pool_storage_account.owner)]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(mut)]
    pub purchase_vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnlockIDO<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: we don't read and write this account
    pub user_token: AccountInfo<'info>,

    pub ido_mint: Account<'info, Mint>,

    #[account(mut)]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    #[account(mut)]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(mut)]
    pub user_vesting: Account<'info, UserVestingAccount>,

    #[account(
        mut,
        seeds = [vesting_storage_account.key().as_ref(), ido_mint.key().as_ref()],
        bump = vesting_storage_account.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
