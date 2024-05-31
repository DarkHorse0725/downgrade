use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };
use std::mem::size_of;

use crate::{ Pool, Staker };

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub stake_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub staker: Account<'info, Staker>,

    #[account(mut, token::mint = stake_mint)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(
      mut,
      token::mint = stake_mint,
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, amount: u64) -> Result<()> {
        // transfer reward token to vault
        let cpi_accounts: Transfer = Transfer {
            from: self.user_token.to_account_info(),
            to: self.stake_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        self.pool.total_staked += amount;

        let clock: Clock = Clock::get()?;
        self.staker.total_staked += amount;
        self.staker.last_update = clock.unix_timestamp;
        msg!("Staked");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub stake_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub staker: Account<'info, Staker>,

    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [b"farm-vault", pool.key().as_ref()],
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

// intialize stake token vault when first deposit
impl<'info> InitVault<'info> {
    pub fn init_vault(&mut self, amount: u64, vault_bump: u8) -> Result<()> {
        // transfer reward token to vault
        let cpi_accounts: Transfer = Transfer {
            from: self.user_token.to_account_info(),
            to: self.stake_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // update staker info
        let clock: Clock = Clock::get()?;
        self.pool.total_staked = amount;
        self.pool.vault_bump = vault_bump;

        self.staker.total_staked = amount;
        self.staker.last_update = clock.unix_timestamp;

        msg!("Init stake");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitStaker<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = signer,
        space = size_of::<Staker>() + 8,
        seeds = [pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub staker: Account<'info, Staker>,

    pub system_program: Program<'info, System>,
}

pub fn init_staker_handler(ctx: Context<InitStaker>) -> Result<()> {
    let pool: &mut Account<Pool> = &mut ctx.accounts.pool;
    let staker: &mut Account<Staker> = &mut ctx.accounts.staker;
    pool.staker_count += 1;
    staker.total_staked = 0;
    staker.withdraw = 0;
    msg!("Init farmer");
    Ok(())
}
