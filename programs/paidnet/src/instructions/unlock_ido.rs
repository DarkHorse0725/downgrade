use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{ self, AssociatedToken, Create },
    token::{ self, Mint, Token, TokenAccount, Transfer },
};

use crate::{ state::{ UserPurchaseAccount, UserVestingAccount }, vesting_logic::calculate_claimable_amount, PoolStorage, VestingStorage };
use crate::error::*;

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
    pub pool_storage_account: Account<'info, PoolStorage>,

    #[account(mut)]
    pub ido_vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnlockIDO<'info> {
    pub fn create_ctx(&self) -> CpiContext<'info, 'info, 'info, 'info, Create<'info>> {
        CpiContext::new(self.associated_token_program.to_account_info(), Create {
            payer: self.signer.to_account_info(),
            associated_token: self.user_token.clone(),
            authority: self.signer.to_account_info(),
            mint: self.ido_mint.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        })
    }
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.ido_vault.to_account_info(),
            to: self.user_token.to_account_info(),
            authority: self.ido_vault.to_account_info(),
        })
    }
}

pub fn unlock_ido_handler(ctx: Context<UnlockIDO>) -> Result<()> {
    let vesting_storage: &Account<VestingStorage> = &ctx.accounts.vesting_storage_account;
    let user_purchase: &Account<UserPurchaseAccount> = &ctx.accounts.user_purchase_account;
    let user_vesting: &Account<UserVestingAccount> = &ctx.accounts.user_vesting;
    if !vesting_storage.claimable {
        return err!(ErrCode::NotClaimable);
    }

    if user_purchase.withdrawn >= user_vesting.total_amount {
        return err!(ErrCode::AlreadyClaimedTotoalAmount);
    }

    if ctx.accounts.user_token.data_is_empty() {
        associated_token::create(ctx.accounts.create_ctx())?;
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
    let seeds: &[&[u8]; 3] = &[
        b"ido-vault",
        ctx.accounts.pool_storage_account.to_account_info().key.as_ref(),
        &[vesting_storage.vault_bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), claimable_amount)?;
    msg!("Unlocked IDO");
    Ok(())
}
