use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{ self, AssociatedToken, Create },
    token::{ self, Mint, Token, TokenAccount, Transfer },
};

use crate::{ Pool, Staker };

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    pub reward_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(mut)]
    pub user_reward_token: AccountInfo<'info>,

    #[account(mut)]
    pub farmer: Account<'info, Staker>,
    #[account(mut)]
    pub farm: Account<'info, Pool>,

    #[account(mut, token::mint = reward_mint)]
    pub reward_pot: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let reward_per_block: u64 = self.farm.reward_per_block;
        let clock: Clock = Clock::get()?;
        let base: u64 = 10;
        let reward: u64 =
            ((clock.unix_timestamp - self.farmer.last_update) as u64) *
            reward_per_block *
            self.farmer.total_staked /
            base.pow((self.farm.farm_decimals) as u32);
        if self.user_reward_token.data_is_empty() {
            let cpi_accounts: Create = Create {
                payer: self.signer.to_account_info(),
                associated_token: self.user_reward_token.clone(),
                authority: self.signer.to_account_info(),
                mint: self.reward_mint.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            };
            let cpi_program: AccountInfo = self.associated_token_program.to_account_info();
            let cpi_ctx: CpiContext<Create> = CpiContext::new(cpi_program, cpi_accounts);
            associated_token::create(cpi_ctx)?;
        }

        if reward > 0 {
            let seeds: &[&[u8]; 3] = &[
                b"reward-pot",
                self.farm.to_account_info().key.as_ref(),
                &[self.farm.pot_bump],
            ];
            let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
            let cpi_accounts: Transfer = Transfer {
                from: self.reward_pot.to_account_info(),
                to: self.user_reward_token.to_account_info(),
                authority: self.reward_pot.to_account_info(),
            };
            let cpi_program: AccountInfo = self.token_program.to_account_info();
            let cpi_ctx: CpiContext<Transfer> = CpiContext::new(
                cpi_program,
                cpi_accounts
            ).with_signer(signer);
            token::transfer(cpi_ctx, reward)?;

            let clock: Clock = Clock::get()?;
            self.farmer.last_update = clock.unix_timestamp;
            self.farmer.withdraw += reward;
        }
        msg!("claimed");
        Ok(())
    }
}
