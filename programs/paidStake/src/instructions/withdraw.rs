use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::{ ErrCode, Pool, Staker };

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub staker: Account<'info, Staker>,

    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,

    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub user_stake_token: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        if self.staker.total_staked < amount {
            return err!(ErrCode::InvalidAmount);
        }
        let seeds: &[&[u8]; 3] = &[
            b"farm-vault",
            self.pool.to_account_info().key.as_ref(),
            &[self.pool.vault_bump],
        ];
        let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
        let cpi_accounts: Transfer = Transfer {
            from: self.stake_vault.to_account_info(),
            to: self.user_stake_token.to_account_info(),
            authority: self.stake_vault.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts).with_signer(
            signer
        );
        token::transfer(cpi_ctx, amount)?;

        let clock: Clock = Clock::get()?;
        self.staker.last_update = clock.unix_timestamp;
        self.staker.total_staked -= amount;
        
        self.pool.total_staked -= amount;
       
        msg!("Withdraw successfully");
        Ok(())
    }
}
