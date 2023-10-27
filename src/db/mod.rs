use num::ToPrimitive;
use rbatis::RBatis;
use rbatis::rbdc::decimal::Decimal;
use crate::db::tables::{AccountEligible, ClaimedAccount, LastSyncBlock, QueryAccount};

pub(crate) mod tables;

pub(crate) async fn upsert_last_sync_block(rb: &mut RBatis, new_block : i64) -> anyhow::Result<()> {
    let block = LastSyncBlock::select_all(rb).await?;
    if block.is_empty() {
        rb.exec("insert into last_sync_block values (?)",
                vec![rbs::to_value!(new_block)])
            .await?;
    } else {
        rb.exec("update last_sync_block set block_number = ?",
                vec![rbs::to_value!(new_block)])
            .await?;
    }
    Ok(())
}

pub async fn get_last_sync_block(rb:&RBatis,start_block: u64) -> anyhow::Result<u64> {
    let block: Vec<LastSyncBlock> = rb
        .query_decode("select block_number from last_sync_block",vec![])
        .await?;
    let number = if block.is_empty() {
        start_block
    } else {
        block[0].block_number.to_u64().unwrap()
    };
    Ok(number)
}

pub(crate) async fn save_query_account(rb: RBatis, query: QueryAccount) -> anyhow::Result<()> {
    println!("query is {:?}",query);
    rb.exec("insert into query_accounts (address,claimable_amount,query_time) \
        values (?,?,?) on conflict(address) do update set claimable_amount = ?,query_time = ?",
            vec![rbs::to_value!(query.address),
                 rbs::to_value![query.claimable_amount.clone()],
                 rbs::to_value!(query.query_time.clone()),
                 rbs::to_value![query.claimable_amount],
                 rbs::to_value!(query.query_time),
            ]).await?;

    Ok(())
}
pub async fn get_all_queried_accounts(rb: &RBatis) ->anyhow::Result<Vec<AccountEligible>> {
    let ret: Vec<QueryAccount> = rb
        .query_decode("select * from query_accounts order by address asc",vec![])
        .await?;
    let accounts_eligible = ret.iter().map(|a| AccountEligible {
        address: a.address.clone(),
        claimable_amount: a.claimable_amount.0.to_string(),
    }).collect::<Vec<_>>();
    Ok(accounts_eligible)
}

pub(crate) async fn save_claimed_accounts(rb: &mut RBatis, accounts: Vec<ClaimedAccount>) -> anyhow::Result<()> {
    for account in accounts {
        rb.exec("insert into claimed_accounts (address,claimed_time,claimed_amount) \
        values (?,?,?) on conflict(address) do nothing",
                vec![rbs::to_value!(account.address),
                     rbs::to_value!(account.claimed_time.clone()),
                     rbs::to_value!(account.claimed_amount.clone()),
                ]).await?;
    }
    Ok(())
}
pub async fn db_get_queried_addresses_number(rb:&RBatis) -> anyhow::Result<u64> {
    let queried_number: u64 = rb
        .query_decode("select count(1) from query_accounts",vec![])
        .await?;
    Ok(queried_number)
}
pub async fn db_get_total_claimed_number(rb:&RBatis) -> anyhow::Result<u64> {
    let claimed_number: u64 = rb
        .query_decode("select count(1) from claimed_accounts",vec![])
        .await?;
    Ok(claimed_number)
}
pub async fn db_get_total_claimed_amount(rb:&RBatis) -> anyhow::Result<Decimal> {
    let claimed_number: Decimal = rb
        .query_decode("select sum(claimed_amount) from claimed_accounts",vec![])
        .await?;
    Ok(claimed_number)
}
#[cfg(test)]
mod test {
    use super::*;
    use ethabi::Uint;
    use web3::types::H160;

   // #[tokio::test]
    // async fn test_update_decimal() {
    //     let rb = Rbatis::new();
    //     let db_url = "postgres://postgres:postgres123@localhost/backend";
    //     rb.init(rbdc_pg::driver::PgDriver {}, db_url).unwrap();
    //     let pool = rb
    //         .get_pool()
    //         .expect("get pool failed");
    //     pool.resize(2);
    //     let reserve_x = Uint::from(12345666);
    //     let reserve_y = Uint::from(666666);
    //     let reserve_x_decimal = Decimal::from_str(&reserve_x.to_string()).unwrap();
    //     let reserve_y_decimal = Decimal::from_str(&reserve_y.to_string()).unwrap();
    //     let pair_address = H160::from_str("0x558038F070A802182355A0FA4807575f30076CeD").unwrap();
    //     println!("{:?}",hex::encode(pair_address));
    //     rb.exec("update pool_info set token_x_reserves = ?,token_y_reserves = ? \
    //         where pair_address = ?", vec![rbs::to_value!(reserve_x_decimal),
    //                                       rbs::to_value!(reserve_y_decimal),
    //                                       //rbs::to_value!("0x558038F070A802182355A0FA4807575f30076CeD")])
    //                                       rbs::Value::String(hex::encode(pair_address))])
    //             .await.unwrap();
    //
    // }
}