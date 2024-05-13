pub mod state;
pub mod error;
pub mod vesting_logic;
pub mod pool_logic;
mod instructions;

use anchor_lang::{
    prelude::*, 
    solana_program::keccak,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{ self, Transfer, TokenAccount, Mint, Token };
use std::mem::size_of;
use state::{ UserPurchaseAccount, UserVestingAccount };
use crate::error::ErrCode;
use crate::state::*;
use anchor_spl::associated_token::{ self, Create };
use self::{
    pool_logic::{ calculate_participiant_fee, MAX_TGE_DATE_ADJUSTMENT, MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS, max_purchase_amount_for_early_access  },
    vesting_logic::calculate_claimable_amount,
};
use instructions::*;

use spl_account_compression::{
    program::SplAccountCompression,
    cpi::{
        accounts:: VerifyLeaf,
        verify_leaf, 
    }
};

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
        let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
        let pool_storage: &mut Account<'_, PoolStorage> = &mut ctx.accounts.pool_storage_account;
        if pool_storage.early_pool_open_time > early_pool_close_time {
            return err!(ErrCode::InvalidTime);
        }

        if early_pool_close_time > open_pool_close_time {
            return err!(ErrCode::InvalidTime);
        }

        if open_pool_close_time > vesting_storage.tge_date {
            return err!(ErrCode::InvalidTime);
        }
        // update time
        pool_storage.early_pool_close_time = early_pool_close_time;
        pool_storage.open_pool_open_time = early_pool_close_time;
        pool_storage.open_pool_close_time = open_pool_close_time;

        Ok(())
    }

    pub fn update_tge_date(ctx: Context<UpdateTGEDate>, tge_date: i64) -> Result<()> {
        let vesting_storage: &mut Account<VestingStorage> = &mut ctx.accounts.vesting_storage_account;
        let pool_storage: &Account<PoolStorage> = &ctx.accounts.pool_storage_account;

        if pool_storage.open_pool_close_time > tge_date {
            return err!(ErrCode::InvalidTime);
        }

        if vesting_storage.tge_update_attempts >= MAX_TGE_DATE_ADJUSTMENT_ATTEMPTS {
            return err!(ErrCode::NotAllowedToAdjustTGEDateExceedsAttempts);
        }

        if tge_date > vesting_storage.initial_tge_date + MAX_TGE_DATE_ADJUSTMENT {
            return err!(ErrCode::NotAllowedToAdjustTGEDateTooFar);
        }

        vesting_storage.tge_date = tge_date;

        Ok(())
    }

    pub fn buy_token_in_early_pool(
        ctx: Context<BuyTokenInEarlyPool>,
        purchase_amount: u64,
        purchase_bump: u8
    ) -> Result<()> {
        let pool_storage: &mut Account<PoolStorage> = &mut ctx.accounts.pool_storage_account;
        // validate time
        let clock: Clock = Clock::get()?;
        if clock.unix_timestamp > pool_storage.early_pool_close_time {
            return err!(ErrCode::TimeOutBuyIDOToken);
        }
        if clock.unix_timestamp < pool_storage.early_pool_open_time {
            return err!(ErrCode::TimeOutBuyIDOToken);
        }
        // validate amount
        if purchase_amount == 0 {
            return err!(ErrCode::InvalidAmount);
        }

        let user_purchase: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;

        let allow_purchase_amount: u64 = max_purchase_amount_for_early_access(
            pool_storage.total_raise_amount,
            pool_storage.open_pool_proportion as u64,
            pool_storage.early_pool_proportion as u64
        );

        if user_purchase.early_purchased + purchase_amount > allow_purchase_amount {
            return err!(ErrCode::ExceedMaxPurchaseAmountForEarlyAccess);
        }

        let participant_fee: u64 = calculate_participiant_fee(
            purchase_amount,
            pool_storage.early_pool_participation_fee_percentage
        );
        let ido_amount: u64 =
            (purchase_amount - participant_fee) * pool_storage.offered_currency.rate;
        let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
        if !vesting_storage.funded {
            return err!(ErrCode::NotFunded);
        }

        // send token to purchase vault
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.user_purchase_token.to_account_info(),
            to: ctx.accounts.purchase_vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, purchase_amount)?;

        // update pool info
        pool_storage.purchase_bump = purchase_bump;
        pool_storage.purchased_amount += purchase_amount;
        // update user vesting info
        let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
        user_vesting.total_amount += ido_amount;
        // update user purchase info
        user_purchase.early_purchased += purchase_amount - participant_fee;
        user_purchase.principal += purchase_amount - participant_fee;
        user_purchase.fee += participant_fee;
        

        Ok(())
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
        let pool_storage: &mut Account<PoolStorage> = &mut ctx.accounts.pool_storage_account;
        // validate time
        let clock: Clock = Clock::get()?;
        if clock.unix_timestamp > pool_storage.open_pool_close_time {
            return err!(ErrCode::TimeOutBuyIDOToken);
        }
        if clock.unix_timestamp < pool_storage.open_pool_open_time {
            return err!(ErrCode::TimeOutBuyIDOToken);
        }
        // validate amount
        if purchase_amount == 0 {
            return err!(ErrCode::InvalidAmount);
        }

        let mut allow_purchase_amount: u64 = pool_storage.max_purchase_amount_for_not_kyc_user;

        if user_type == "KYC_USER" {
            // verfify leaf
            let leaf: [u8; 32] =
            keccak::hashv(&[note.as_bytes(), pool_storage.owner.as_ref()]).to_bytes();
            let merkle_tree: Pubkey = ctx.accounts.merkle_tree.key();
            let signer_seeds: &[&[&[u8]]] = &[&[
            merkle_tree.as_ref(), // The address of the merkle tree account as a seed
            &[*ctx.bumps.get("tree_authority").unwrap()], // The bump seed for the pda
        ]];
            let cpi_ctx: CpiContext<VerifyLeaf> = CpiContext::new_with_signer(
                ctx.accounts.compression_program.to_account_info(), // The spl account compression program
                VerifyLeaf {
                    merkle_tree: ctx.accounts.merkle_tree.to_account_info(), // The merkle tree account to be modified
                },
                signer_seeds, // The seeds for pda signing
            );
            // Verify or Fails
            verify_leaf(cpi_ctx, root, leaf, index)?;
            allow_purchase_amount = pool_storage.max_purchase_amount_for_kyc_user;
        }

        let user_purchase: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;
        if user_purchase.early_purchased + purchase_amount > allow_purchase_amount {
            return err!(ErrCode::ExceedMaxPurchaseAmountForEarlyAccess);
        }

        let participant_fee = calculate_participiant_fee(
            purchase_amount,
            pool_storage.early_pool_participation_fee_percentage
        );
        let ido_amount: u64 =
            (purchase_amount - participant_fee) * pool_storage.offered_currency.rate;
        let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
        if !vesting_storage.funded {
            return err!(ErrCode::NotFunded);
        }

        // send token to purchase vault
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.user_purchase_token.to_account_info(),
            to: ctx.accounts.purchase_vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, purchase_amount)?;

        // update pool info
        pool_storage.purchase_bump = purchase_bump;
        pool_storage.purchased_amount += purchase_amount;
        // update user vesting info
        let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
        user_vesting.total_amount += ido_amount;
        // update user purchase info
        
        user_purchase.principal += purchase_amount - participant_fee;
        user_purchase.fee += participant_fee;
        

        Ok(())
    }

    pub fn fund_ido_token(ctx: Context<FundIDO>, amount: u64, bump: u8) -> Result<()> {
        // transfer token
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = ctx.accounts.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        // update info
        let vesting_storage: &mut Account<
            '_,
            VestingStorage
        > = &mut ctx.accounts.vesting_storage_account;
        vesting_storage.ido_token = ctx.accounts.ido_mint.key();
        vesting_storage.funded = true;
        vesting_storage.vault_bump = bump;
        vesting_storage.total_funded_amount += amount;
        Ok(())
    }

    // when failed
    pub fn withdraw_ido_token(ctx: Context<WithdrawIDOToken>, amount: u64) -> Result<()> {
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
        token::transfer(cpi_ctx, amount)?;
        Ok(())
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
pub struct FundIDO<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    pub ido_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [vesting_storage_account.key().as_ref(), ido_mint.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = ido_mint,
        token::authority = vault
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawIDOToken<'info> {
    #[account(mut, constraint = signer.key() == vesting_storage_account.owner)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    pub ido_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [vesting_storage_account.key().as_ref(), ido_mint.key().as_ref()],
        bump = vesting_storage_account.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vesting_storage", signer.key().as_ref()],
        bump
    )]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UpdateTime<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = pool_storage_account.owner == signer.key()
    )]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(constraint = vesting_storage_account.owner == signer.key())]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTGEDate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(constraint = pool_storage_account.owner == signer.key())]
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account( 
        mut,
        constraint = vesting_storage_account.owner == signer.key()
    )]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub system_program: Program<'info, System>,
}

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
        init_if_needed,
        payer = signer,
        seeds = [pool_storage_account.key().as_ref(), purchase_mint.key().as_ref()],
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
        space = size_of::<UserPurchaseAccount>() +8,
        seeds = [pool_storage_account.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<UserVestingAccount>() + 8,
        seeds = [ido_mint.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_vesting: Account<'info, UserVestingAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct BuyTokenInOpenPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub ido_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_storage_account: Account<'info, PoolStorage>,
    pub vesting_storage_account: Account<'info, VestingStorage>,

    #[account(mut)]
    pub user_purchase_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub purchase_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_purchase_account: Account<'info, UserPurchaseAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<UserVestingAccount>() + 8,
        seeds = [ido_mint.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_vesting: Account<'info, UserVestingAccount>,
    /// CHECK: This account is validated by the spl account compression program
    pub merkle_tree: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
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
