use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };
use std::mem::size_of;

use crate::{ Pool, Staker };

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub stake_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<Staker>() + 8,
        seeds = [pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub staker: Account<'info, Staker>,

    #[account(mut, token::mint = stake_mint)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"stake-vault", pool.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = stake_mint,
        token::authority = stake_vault
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_token.to_account_info(),
            to: self.stake_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

// @stake paid by staker
pub fn stake_handler(ctx: Context<Stake>, amount: u64, bump: u8) -> Result<()> {
    // transfer token to vault
    token::transfer(ctx.accounts.transfer_ctx(), amount)?;
    let pool: &mut Box<Account<Pool>> = &mut ctx.accounts.pool;
    // update staker info
    let staker: &mut Account<Staker> = &mut ctx.accounts.staker;
    pool.total_staked += amount;
    pool.vault_bump = bump;

    let clock: Clock = Clock::get()?;
    staker.total_staked += amount;
    staker.last_update = clock.unix_timestamp;
    msg!("Staked");
    Ok(())
}
