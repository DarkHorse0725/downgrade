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

    #[account(has_one = owner, constraint = reward_mint.key() == pool.reward_mint)]
    pub pool: Account<'info, Pool>,

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
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.owner_token.to_account_info(),
            to: self.reward_pot.to_account_info(),
            authority: self.owner.to_account_info(),
        })
    }
}

pub fn add_reward_handler(ctx: Context<AddReward>, amount: u64) -> Result<()> {
  token::transfer(ctx.accounts.transfer_ctx(), amount)?;
    msg!("Init reward");
    Ok(())
}
