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

    // @dev ido token account of owner
    #[account(mut)]
    pub owner_offer_token: Account<'info, TokenAccount>,

    // @dev mint address of ido token
    pub offer_mint: Box<Account<'info, Mint>>,

    // @dev offer vault
    #[account(
        mut,
        token::mint = offer_mint
    )]
    pub offer_vault: Account<'info, TokenAccount>,

    // @dev pool account
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

// @dev allowed to withdraw ido token by creator if failed
pub fn withdraw_offer_handler(ctx: Context<WithdrawOffer>, amount: u64) -> Result<()> {
    let bump: u8 = ctx.accounts.pool.offered_bump;
    // seed of authority pda of offer vault
    let seeds: &[&[u8]; 3] = &[
        b"offer-vault",
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    // transfer token to creator token account
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;
    msg!("Withdraw ido token");
    Ok(())
}
