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

    // @dev mint address of reward token
    pub reward_mint: Box<Account<'info, Mint>>,
    // @dev reward token account of user
    /// CHECK:
    #[account(mut)]
    pub user_reward_token: AccountInfo<'info>,

    // @dev staker account
    #[account(mut)]
    pub staker: Box<Account<'info, Staker>>,
    // pool account
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    // @dev reward pot
    #[account(mut, token::mint = reward_mint)]
    pub reward_pot: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn create_ctx(&self) -> CpiContext<'info, 'info, 'info, 'info, Create<'info>> {
        CpiContext::new(self.associated_token_program.to_account_info(), Create {
            payer: self.signer.to_account_info(),
            associated_token: self.user_reward_token.clone(),
            authority: self.signer.to_account_info(),
            mint: self.reward_mint.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        })
    }
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.reward_pot.to_account_info(),
            to: self.user_reward_token.to_account_info(),
            authority: self.reward_pot.to_account_info(),
        })
    }
}

// @dev claim reward by staker
pub fn claim_handler(ctx: Context<Claim>) -> Result<()> {
    // calculate reward amount
    let reward_per_block: u64 = ctx.accounts.pool.reward_per_block;
    let clock: Clock = Clock::get()?;
    let base: u64 = 10;
    let reward: u64 =
        (((clock.unix_timestamp - ctx.accounts.staker.last_update) as u64) *
            reward_per_block *
            ctx.accounts.staker.total_staked) /
        base.pow(ctx.accounts.pool.stake_decimals as u32);
    if ctx.accounts.user_reward_token.data_is_empty() {
        associated_token::create(ctx.accounts.create_ctx())?;
    }

    // transfer reward to user
    if reward > 0 {
        let seeds: &[&[u8]; 3] = &[
            b"reward-pot",
            ctx.accounts.pool.to_account_info().key.as_ref(),
            &[ctx.accounts.pool.pot_bump],
        ];
        let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
        token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), reward)?;

        let clock: Clock = Clock::get()?;
        let staker = &mut ctx.accounts.staker;
        staker.last_update = clock.unix_timestamp;
        staker.withdraw += reward;
    }
    msg!("claimed");
    Ok(())
}
