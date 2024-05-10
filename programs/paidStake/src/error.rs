use anchor_lang::prelude::*;

#[error_code]
pub enum ErrCode {
    #[msg("Invalid amount")]
    InvalidAmount,
}