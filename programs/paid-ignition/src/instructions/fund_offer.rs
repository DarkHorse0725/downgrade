use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ self, Mint, Token, TokenAccount, Transfer },
};

use crate::Pool;

#[derive(Accounts)]
pub struct FundOffer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // mint address of ido token
    pub offer_mint: Box<Account<'info, Mint>>,

    // ido token account of owner
    #[account(
      mut,
      token::mint = offer_mint,
    )]
    pub owner_token: Account<'info, TokenAccount>,

    // pool account
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    // offer vault
    #[account(
        init_if_needed,
        payer = owner,
        seeds = [b"offer-vault", pool.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = offer_mint,
        token::authority = offer_vault
    )]
    pub offer_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundOffer<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.owner_token.to_account_info(),
            to: self.offer_vault.to_account_info(),
            authority: self.owner.to_account_info(),
        })
    }
}
// @dev allowed to deposit ido token by creator after creating pool
pub fn fund_offer_handler(ctx: Context<FundOffer>, amount: u64, bump: u8) -> Result<()> {
    // transfer token
    token::transfer(ctx.accounts.transfer_ctx(), amount)?;
    // update info
    let pool: &mut Box<Account<Pool>> = &mut ctx.accounts.pool;
    pool.offered_currency.mint = ctx.accounts.offer_mint.key();
    pool.funded = true;
    pool.offered_bump = bump;
    pool.total_funded_amount += amount;
    msg!("Funded IDO");
    Ok(())
}
