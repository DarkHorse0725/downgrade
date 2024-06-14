use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{ self, AssociatedToken, Create },
    token::{ self, Mint, Token, TokenAccount, Transfer },
};

use crate::{ calculate_claimable_amount, error::ErrCode, Buyer, Pool };

#[derive(Accounts)]
pub struct ClaimOffer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: we don't read and write this account
    #[account(mut)]
    pub user_token: AccountInfo<'info>,

    pub offer_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub buyer: Box<Account<'info, Buyer>>,

    #[account(
        mut,
        seeds = [b"offer-vault", pool.key().as_ref()],
        bump,
        rent_exempt = enforce,
        token::mint = offer_mint,
    )]
    pub offer_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimOffer<'info> {
    pub fn create_ctx(&self) -> CpiContext<'info, 'info, 'info, 'info, Create<'info>> {
        CpiContext::new(self.associated_token_program.to_account_info(), Create {
            payer: self.signer.to_account_info(),
            associated_token: self.user_token.clone(),
            authority: self.signer.to_account_info(),
            mint: self.offer_mint.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        })
    }
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.offer_vault.to_account_info(),
            to: self.user_token.to_account_info(),
            authority: self.offer_vault.to_account_info(),
        })
    }
}

// @dev allowed to unlock ido token by user after success
pub fn claim_offer_handler(ctx: Context<ClaimOffer>) -> Result<()> {
    let buyer: &Box<Account<Buyer>> = &ctx.accounts.buyer;
    // check if allowed to claim
    if !ctx.accounts.pool.claimable {
        return err!(ErrCode::NotClaimable);
    }

    // check if has claimable amount
    if buyer.cliamed_amount >= buyer.total_amount {
        return err!(ErrCode::AlreadyClaimedTotoalAmount);
    }

    // create token account if empty
    if ctx.accounts.user_token.data_is_empty() {
        associated_token::create(ctx.accounts.create_ctx())?;
    }
    // check vesting amount
    let now: i64 = ctx.accounts.clock.unix_timestamp as i64;
    let claimable_amount: u64 = calculate_claimable_amount(
        buyer.total_amount,
        buyer.cliamed_amount,
        ctx.accounts.pool.tge_percentage,
        ctx.accounts.pool.tge_date,
        ctx.accounts.pool.vesting_cliff,
        ctx.accounts.pool.vesting_frequency as u64,
        ctx.accounts.pool.number_of_vesting as u64,
        now
    );

    if claimable_amount == 0 {
        return err!(ErrCode::NotclaimableAmount);
    }
    // seeds of authority pda of offer vault
    let seeds: &[&[u8]; 3] = &[
        b"ido-vault",
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.offered_bump],
    ];
    let signer: &[&[&[u8]]; 1] = &[&seeds[..]];
    //   transfer token to user token account
    token::transfer(ctx.accounts.transfer_ctx().with_signer(signer), claimable_amount)?;
    // send ido token to user
    // let pool: &mut Box<Account<Pool>> = &mut ctx.accounts.pool;
    msg!("Unlocked IDO");
    Ok(())
}
