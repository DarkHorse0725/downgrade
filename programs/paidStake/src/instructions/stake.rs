use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };
use std::mem::size_of;

use crate::{ Pool, Staker };

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub farm_mint: Account<'info, Mint>,
    #[account(mut)]
    pub farm: Account<'info, Pool>,

    #[account(mut)]
    pub farmer: Account<'info, Staker>,

    #[account(mut, token::mint = farm_mint)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(
      mut,
      token::mint = farm_mint,
    )]
    pub farm_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, amount: u64) -> Result<()> {
        // transfer reward token to vault
        let cpi_accounts: Transfer = Transfer {
            from: self.user_token.to_account_info(),
            to: self.farm_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        self.farm.total_staked += amount;

        let clock: Clock = Clock::get()?;
        self.farmer.total_staked += amount;
        self.farmer.last_update = clock.unix_timestamp;
        msg!("Staked");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub farm_mint: Account<'info, Mint>,
    #[account(mut)]
    pub farm: Account<'info, Pool>,

    #[account(mut)]
    pub farmer: Account<'info, Staker>,

    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [b"farm-vault", farm.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = farm_mint,
        token::authority = farm_vault
    )]
    pub farm_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitVault<'info> {
    pub fn init_vault(&mut self, amount: u64, vault_bump: u8) -> Result<()> {
        // transfer reward token to vault
        let cpi_accounts: Transfer = Transfer {
            from: self.user_token.to_account_info(),
            to: self.farm_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let clock: Clock = Clock::get()?;
        self.farm.total_staked = amount;
        self.farm.vault_bump = vault_bump;

        self.farmer.total_staked = amount;
        self.farmer.last_update = clock.unix_timestamp;

        msg!("Init stake");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitStaker<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub farm: Account<'info, Pool>,

    #[account(
        init,
        payer = signer,
        space = size_of::<Staker>() + 8,
        seeds = [farm.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub farmer: Account<'info, Staker>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitStaker<'info> {
    pub fn init_staker(&mut self) -> Result<()> {
        self.farm.farmer_count += 1;
        self.farmer.total_staked = 0;
        self.farmer.withdraw = 0;
        msg!("Init farmer");
        Ok(())
    }
}
