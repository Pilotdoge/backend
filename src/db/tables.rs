use rbatis::rbdc::decimal::Decimal;
use std::str::FromStr;
use web3::types::H160;
use crate::watcher::event::ClaimEvent;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LastSyncBlock {
    pub block_number: i64,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct QueryAccount {
    pub address: String,
    pub claimable_amount: Decimal,
    pub query_time: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountEligible {
    pub address: String,
    pub claimable_amount: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClaimedAccount {
    pub address: String,
    pub claimed_time: i64,
    pub claimed_amount: Decimal,
}

rbatis::crud!(QueryAccount {}, "query_accounts");
rbatis::crud!(ClaimedAccount {}, "claimed_accounts");
rbatis::crud!(LastSyncBlock {}, "last_sync_block");

impl Default for QueryAccount {
    fn default() -> Self {
        QueryAccount {
            address: H160::zero().to_string(),
            claimable_amount: Decimal::from_str("0").unwrap(),
            query_time: 0,
        }
    }
}
impl From<ClaimEvent> for ClaimedAccount {
    fn from(event: ClaimEvent) -> Self {
        Self {
            address: event.address.to_string(),
            claimed_time: event.claimed_time.as_u64() as i64,
            claimed_amount: Decimal::from_str(&event.amount.to_string()).unwrap_or(Decimal::from_str("0").unwrap()),
        }
    }
}
