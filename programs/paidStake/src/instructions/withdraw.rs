use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::{ ErrCode, Pool, Staker };

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // @dev pool account
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    // @dev staker account
    #[account(mut)]
    pub staker: Box<Account<'info, Staker>>,

    // @dev mint address of stake token
    pub stake_mint: Box<Account<'info, Mint>>,

    // @dev stake vault
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(mut)]
    pub user_stake_token: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.stake_vault.to_account_info(),
            to: self.user_stake_token.to_account_info(),
            authority: self.stake_vault.to_account_info(),
        })
    }
}

// @dev withdraw token by staker
pub fn withdraw_handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    if ctx.accounts.staker.total_staked < amount {
        return err!(ErrCode::InvalidAmount);
    }
    // seeds of authority pda of stake vault
    let seeds: &[&[u8]; 3] = &[
        b"stake-vault",
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.vault_bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    // transfer token to user token account
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), amount)?;

    // update staker info
    let clock: Clock = Clock::get()?;
    let staker: &mut Account<Staker> = &mut ctx.accounts.staker;
    staker.last_update = clock.unix_timestamp;
    staker.total_staked -= amount;
    let pool: &mut Account<Pool> = &mut ctx.accounts.pool;
    pool.total_staked -= amount;
    msg!("Withdraw successfully");
    Ok(())
}
