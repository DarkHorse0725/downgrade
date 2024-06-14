use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    // @dev pool owner 
    pub owner: Pubkey,
    // @dev info of purchase token
    pub purchase_currency: PurchaseCurrency,
    // @dev info of ido token
    pub offered_currency: OfferedCurrency,
    // @dev max purchase amount per buyers in early pool
    pub max_purchase_amount_for_early_access: u64,
    // @dev max purchase amout per kyc users in open pool, based on purchase token
    pub max_purchase_amount_for_kyc_user: u64,
    // @dev max purchase amount per not kyc users in open pool, based on purchase token
    pub max_purchase_amount_for_not_kyc_user: u64,
    // @dev token fee percentage of creator 
    pub token_fee_percentage: u16,
    // @dev it allows to claim fee by creator
    pub token_fee_cliamed_status: bool,
    // @dev participant fee of buyer in early pool
    pub early_pool_participation_fee_percentage: u16,
    // @dev participant fee of buyer in open pool
    pub open_pool_participation_fee_percentage: u16,
    // @dev share of open pool, based on ido token
    pub open_pool_proportion: u16,
    // @dev share of early pool, based on ido token
    pub early_pool_proportion: u16,
    // @dev total raising target(hardcap), based on purchase token
    pub total_raise_amount: u64,
    // @dev start unix time of early pool
    pub early_pool_open_time: i64,
    // @dev end unix time of early pool
    pub early_pool_close_time: i64,
    // @dev start unix time of open pool
    pub open_pool_open_time: i64,
    // @dev end unix time of open pool
    pub open_pool_close_time: i64,
    // @dev purchased amount in open pool, based on purchase token
    pub purchased_amount_in_open_pool: u64,
    // @dev purchased amount in early pool, based on purchase token
    pub purchased_amount_in_early_access: u64,
    // @dev total purchased amount, based on purchase token
    pub purchased_amount: u64,
    // @dev claimed amount of ido token after success
    pub fund_claimed_amount: u64,
    // @dev unix time of tge date
    pub tge_date: i64,
    // @dev tge percentage of ido token
    pub tge_percentage: u16,
    // @dev vesting cliff
    pub vesting_cliff: i64,
    // @dev vesting frequency
    pub vesting_frequency: i64,
    // @dev number of vesting release
    pub number_of_vesting: i64,
    // @dev total funded amount of ido token
    pub total_funded_amount: u64,
    // @dev true if collaborator fund enough ido token
    pub funded: bool,
    // @dev true if creator allow for user to claim
    pub claimable: bool,
    // @dev true if creator cancelled
    pub emergency_cancelled: bool,
    // @dev true if private sale
    pub private_raise: bool,
    // @dev bump for authority pda of purchase token account
    pub purchase_bump: u8,
    // @dev bump for authority pad of offered token account
    pub offered_bump: u8,
    // @dev allowed updated attempts
    pub tge_update_attempts: u8,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OfferedCurrency {
    // @dev amount of ido token for 1 purchase token
    pub rate: u64,
    // @dev decimals of ido token
    pub decimals: u8,
    // @dev mint address of ido token
    pub mint: Pubkey,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PurchaseCurrency {
    // @dev decimals of purchase token
    pub decimals: u8,
    // @dev mint address of purchase token
    pub mint: Pubkey,
}


