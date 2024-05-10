use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::Pool;

#[derive(Accounts)]
pub struct AddReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub reward_mint: Account<'info, Mint>,

    #[account(mut, token::mint = reward_mint, token::authority = owner)]
    pub owner_token: Account<'info, TokenAccount>,

    #[account(
      has_one = owner,
      constraint = reward_mint.key() == farm.reward_mint
    )]
    pub farm: Account<'info, Pool>,

    #[account(
      mut,
      token::mint = reward_mint,
      token::authority = reward_pot
    )]
    pub reward_pot: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddReward<'info> {
    pub fn add_reward(&mut self, amount: u64) -> Result<()> {
        // transfer reward token to vault
        let cpi_accounts: Transfer = Transfer {
            from: self.owner_token.to_account_info(),
            to: self.reward_pot.to_account_info(),
            authority: self.owner.to_account_info(),
        };
        let cpi_program: AccountInfo = self.token_program.to_account_info();
        let cpi_ctx: CpiContext<Transfer> = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        msg!("Init reward");
        Ok(())
    }
}
