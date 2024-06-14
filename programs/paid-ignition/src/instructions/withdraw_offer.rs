use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ self, Mint, Token, TokenAccount, Transfer },
};

use crate::Pool;

#[derive(Accounts)]
pub struct WithdrawOffer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub owner_offer_token: Account<'info, TokenAccount>,

    pub offer_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = offer_mint
    )]
    pub offer_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawOffer<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.offer_vault.to_account_info(),
            to: self.owner_offer_token.to_account_info(),
            authority: self.offer_vault.to_account_info(),
        })
    }
}

// withdraw ido token by creator after failed
pub fn withdraw_offer_handler(ctx: Context<WithdrawOffer>, amount: u64) -> Result<()> {
    let bump: u8 = ctx.accounts.pool.offered_bump;
    let seeds: &[&[u8]; 3] = &[
        b"offer-vault",
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;
    msg!("Withdraw ido token");
    Ok(())
}
