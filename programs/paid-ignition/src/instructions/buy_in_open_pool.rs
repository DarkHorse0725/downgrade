use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, Token, TokenAccount, Transfer };

use crate::{ calculate_participiant_fee, error::ErrCode, Buyer, Pool };
use std::mem::size_of;

#[derive(Accounts)]
pub struct BuyInOpenPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // @dev mint address of purchase token
    pub purchase_mint: Box<Account<'info, Mint>>,

    // @dev user purchase token account
    #[account(
      mut,
      token::mint = purchase_mint,
    )]
    pub user_purchase_token: Account<'info, TokenAccount>,

    // @dev pool account
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    // @dev purchase token vault
    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"purchase-vault", pool.key().as_ref()],
        bump,
        owner = token_program.key(),
        rent_exempt = enforce,
        token::mint = purchase_mint,
        token::authority = purchase_vault
    )]
    pub purchase_vault: Account<'info, TokenAccount>,

    // @dev buyer account
    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<Buyer>() + 8,
        seeds = [b"buyer", pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub buyer: Box<Account<'info, Buyer>>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyInOpenPool<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_purchase_token.to_account_info(),
            to: self.purchase_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        })
    }
}

// @dev allow to buy token by user in open pool
pub fn buy_in_open_pool_handler(
    ctx: Context<BuyInOpenPool>,
    purchase_amount: u64,
    bump: u8
) -> Result<()> {
    // validate time
    let clock: Clock = Clock::get()?;
    if clock.unix_timestamp > ctx.accounts.pool.open_pool_close_time {
        return err!(ErrCode::TimeOutBuyIDOToken);
    }
    if clock.unix_timestamp < ctx.accounts.pool.open_pool_open_time {
        return err!(ErrCode::TimeOutBuyIDOToken);
    }
    // validate amount
    if purchase_amount == 0 {
        return err!(ErrCode::InvalidAmount);
    }

    // let mut allow_purchase_amount: u64 = pool_storage.max_purchase_amount_for_not_kyc_user;

    // if user_type == "KYC_USER" {
    //     // verfify leaf
    //     let leaf: [u8; 32] = keccak
    //         ::hashv(&[note.as_bytes(), pool_storage.owner.as_ref()])
    //         .to_bytes();
    //     let merkle_tree: Pubkey = ctx.accounts.merkle_tree.key();
    //     let signer_seeds: &[&[&[u8]]] = &[
    //         &[
    //             merkle_tree.as_ref(), // The address of the merkle tree account as a seed
    //             &[*ctx.bumps.get("tree_authority").unwrap()], // The bump seed for the pda
    //         ],
    //     ];
    //     let cpi_ctx: CpiContext<VerifyLeaf> = CpiContext::new_with_signer(
    //         ctx.accounts.compression_program.to_account_info(), // The spl account compression program
    //         VerifyLeaf {
    //             merkle_tree: ctx.accounts.merkle_tree.to_account_info(), // The merkle tree account to be modified
    //         },
    //         signer_seeds // The seeds for pda signing
    //     );
    //     // Verify or Fails
    //     verify_leaf(cpi_ctx, root, leaf, index)?;
    //     allow_purchase_amount = pool_storage.max_purchase_amount_for_kyc_user;
    // }

    // let early_purchased: u64 = ctx.accounts.user_purchase_account.early_purchased;
    // if early_purchased + purchase_amount > allow_purchase_amount {
    //     return err!(ErrCode::ExceedMaxPurchaseAmountForEarlyAccess);
    // }

    // calculate fee amount
    let participant_fee: u64 = calculate_participiant_fee(
        purchase_amount,
        ctx.accounts.pool.early_pool_participation_fee_percentage
    );
    let ido_amount: u64 = (purchase_amount - participant_fee) * ctx.accounts.pool.offered_currency.rate;
    if !ctx.accounts.pool.funded {
        return err!(ErrCode::NotFunded);
    }

    // send token to purchase vault
    token::transfer(ctx.accounts.transfer_ctx(), purchase_amount)?;
    // update pool info
    let pool: &mut Box<Account<Pool>> =&mut ctx.accounts.pool;
    pool.purchase_bump = bump;
    pool.purchased_amount += purchase_amount;
    // update user vesting info
    let buyer = &mut ctx.accounts.buyer;
    buyer.total_amount += ido_amount;
    // update user purchase info
    buyer.total_purchase += purchase_amount - participant_fee;
    msg!("Bought token");
    Ok(())
}
