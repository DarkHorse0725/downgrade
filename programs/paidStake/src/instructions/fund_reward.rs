use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::Pool;

#[derive(Accounts)]
pub struct FundReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(mut,token::mint = reward_mint)]
    pub owner_token: Account<'info, TokenAccount>,

    #[account(
      mut, 
      constraint = reward_mint.key() == pool.reward_mint
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"reward-pot", pool.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = reward_mint,
        token::authority = reward_pot
    )]
    pub reward_pot: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundReward<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.owner_token.to_account_info(),
            to: self.reward_pot.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

// @dev allowed to deposit token
pub fn fund_reward_handler(
    ctx: Context<FundReward>,
    amount: u64, 
    pot_bump: u8,
) -> Result<()> {
    // transfer token to reward pot
    token::transfer(ctx.accounts.transfer_ctx(), amount)?;
    // update pool info
    let pool: &mut Account<Pool> = &mut ctx.accounts.pool;
    pool.pot_bump = pot_bump;
    msg!("Init reward");
    Ok(())
}
