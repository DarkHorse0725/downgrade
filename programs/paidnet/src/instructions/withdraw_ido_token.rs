use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token::{ self, Mint, Token, TokenAccount, Transfer } };

use crate::{PoolStorage, VestingStorage};

#[derive(Accounts)]
pub struct WithdrawIDOToken<'info> {
    #[account(mut, constraint = signer.key() == vesting_storage_account.owner)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    pub ido_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = ido_mint
    )]
    pub ido_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vesting_storage_account: Account<'info, VestingStorage>,

    pub pool_storage_account: Account<'info, PoolStorage>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawIDOToken<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.ido_vault.to_account_info(),
            to: self.user_token.to_account_info(),
            authority: self.ido_vault.to_account_info(),
        })
    }
}

pub fn withdraw_ido_handler(ctx: Context<WithdrawIDOToken>, amount: u64) -> Result<()> {
    let bump: u8 = ctx.accounts.vesting_storage_account.vault_bump;
    let seeds: &[&[u8]; 3]= &[
        b"ido-vault",
        ctx.accounts.pool_storage_account.to_account_info().key.as_ref(),
        &[bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;
    msg!("Withdraw ido token");
    Ok(())
}
