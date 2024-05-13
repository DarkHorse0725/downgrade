use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token::{ self, Mint, Token, TokenAccount, Transfer } };

use crate::{PoolStorage, VestingStorage};

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
        seeds = [b"ido-vault", pool_storage_account.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = ido_mint,
        token::authority = ido_vault
    )]
    pub ido_vault: Account<'info, TokenAccount>,

    pub pool_storage_account: Account<'info, PoolStorage>,
    #[account(mut)]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundIDO<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_token.to_account_info(),
            to: self.ido_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

pub fn fund_ido_handler(ctx: Context<FundIDO>, amount: u64, bump: u8) -> Result<()> {
    // transfer token
    token::transfer(ctx.accounts.transfer_ctx(), amount)?;
    // update info
    let vesting_storage: &mut Account<VestingStorage> = &mut ctx.accounts.vesting_storage_account;
    vesting_storage.ido_token = ctx.accounts.ido_mint.key();
    vesting_storage.funded = true;
    vesting_storage.vault_bump = bump;
    vesting_storage.total_funded_amount += amount;
    msg!("Funded IDO");
    Ok(())
}
