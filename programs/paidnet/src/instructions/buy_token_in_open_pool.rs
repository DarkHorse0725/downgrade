use anchor_lang::{ prelude::*, solana_program::keccak };
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };
use std::mem::size_of;
use crate::pool_logic::calculate_participiant_fee;
use crate::{ PoolStorage, VestingStorage, UserPurchaseAccount, UserVestingAccount };
use crate::error::*;
use spl_account_compression::{
    program::SplAccountCompression,
    cpi::{ accounts::VerifyLeaf, verify_leaf },
};

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
        seeds = [b"user-vesting", pool_storage_account.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub user_vesting: Account<'info, UserVestingAccount>,
    // The merkle tree account
    /// CHECK: This account is validated by the spl account compression program
    // #[account(mut)]
    // pub merkle_tree: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyTokenInOpenPool<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_purchase_token.to_account_info(),
            to: self.purchase_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

pub fn buy_token_in_open_pool_handler(
    ctx: Context<BuyTokenInOpenPool>,
    purchase_amount: u64,
    // index: u32,
    // root: [u8; 32],
    // note: String,
    // user_type: String,
    purchase_bump: u8
) -> Result<()> {
    let pool_storage: &Account<PoolStorage> = &ctx.accounts.pool_storage_account;
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

    // if user_type == "KYC_USER" {
    //     // verfify leaf
    //     let leaf: [u8; 32] = keccak
    //         ::hashv(&[note.as_bytes(), pool_storage.owner.as_ref()])
    //         .to_bytes();
    //     let merkle_tree: Pubkey = ctx.accounts.merkle_tree.key();
    //     let signer_seeds: &[&[&[u8]]] = &[
    //         &[
    //             merkle_tree.as_ref(), // The address of the merkle tree account as a seed
    //             &[*ctx.bumps.get("tree_authority").unwrap()], // The bump seed for the pda
    //         ],
    //     ];
    //     let cpi_ctx: CpiContext<VerifyLeaf> = CpiContext::new_with_signer(
    //         ctx.accounts.compression_program.to_account_info(), // The spl account compression program
    //         VerifyLeaf {
    //             merkle_tree: ctx.accounts.merkle_tree.to_account_info(), // The merkle tree account to be modified
    //         },
    //         signer_seeds // The seeds for pda signing
    //     );
    //     // Verify or Fails
    //     verify_leaf(cpi_ctx, root, leaf, index)?;
    //     allow_purchase_amount = pool_storage.max_purchase_amount_for_kyc_user;
    // }

    let early_purchased: u64 = ctx.accounts.user_purchase_account.early_purchased;
    if early_purchased + purchase_amount > allow_purchase_amount {
        return err!(ErrCode::ExceedMaxPurchaseAmountForEarlyAccess);
    }

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
    token::transfer(ctx.accounts.transfer_ctx(), purchase_amount)?;
    // update pool info
    let pool: &mut Account<PoolStorage> = &mut ctx.accounts.pool_storage_account;
    pool.purchase_bump = purchase_bump;
    pool.purchased_amount += purchase_amount;
    // update user vesting info
    let user_vesting: &mut Account<UserVestingAccount> = &mut ctx.accounts.user_vesting;
    user_vesting.total_amount += ido_amount;
    // update user purchase info
    let user: &mut Account<UserPurchaseAccount> = &mut ctx.accounts.user_purchase_account;
    user.principal += purchase_amount - participant_fee;
    user.fee += participant_fee;
    msg!("Bought token");
    Ok(())
}
