use anchor_lang::prelude::*;

declare_id!("8FcYYJ38nxLKWD8BN6JYs8b3yFnnXzkrL9Pfx43NNUPj");

#[program]
pub mod paid_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
