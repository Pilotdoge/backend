pub mod server;
pub mod config;
pub mod route;
pub mod db;
pub mod watcher;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use dotenvy::dotenv;
use crate::config::Config;
use crate::server::AppState;
use futures::executor::block_on;
use futures::channel::mpsc;
use futures::SinkExt;
use futures::StreamExt;
use merkle_tree_rs::standard::StandardMerkleTree;
use rbatis::RBatis;
use crate::db::tables::QueryAccount;
use crate::watcher::watcher::run_watcher;

pub fn init_db(db_url:String,pool_size: usize) -> RBatis {
    let rb = RBatis::new();
    rb.init(rbdc_pg::driver::PgDriver {}, &db_url).unwrap();
    let pool = rb
        .get_pool()
        .expect("get pool failed");
    pool.resize(pool_size);
    log::info!("postgres database init ok!");
    return rb;
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect("Config file not found");
    env_logger::init();
    let config = Config::from_env();
    let rb = init_db(config.database_url.clone(), config.db_pool_size as usize);
    let accounts_eligible = db::get_all_queried_accounts(&rb)
        .await.expect("get all queried accounts from db failed");
    let tree_values = accounts_eligible.iter()
        .map(|ae| vec![ae.address.clone(),ae.claimable_amount.clone()])
        .collect::<Vec<_>>();
    //because the merkle proof should be made from index 1,we add a zero account on the index 0 of tree node
    if config.claim_start && tree_values.is_empty() {
        let zero_account = QueryAccount::default();
        db::save_query_account(rb.clone(),zero_account).await.expect("add zero account failed!");
    }
    let eligible_tree = if config.claim_start {
        let tree: StandardMerkleTree = StandardMerkleTree::of(tree_values, &["address".to_string(), "uint256".to_string()]);
        Some(Arc::new(Mutex::new(tree)))
    } else {
        None
    };

    let app_state = AppState {
        config:config.clone(),
        db: rb.clone(),
        eligible_tree,
    };
    server::run_server(app_state).await;

    let watcher_handler = run_watcher(config.clone(),rb.clone()).await;

    // handle ctrl+c
    let (stop_signal_sender, mut stop_signal_receiver) = mpsc::channel(256);
    {
        let stop_signal_sender = RefCell::new(stop_signal_sender.clone());
        ctrlc::set_handler(move || {
            let mut sender = stop_signal_sender.borrow_mut();
            block_on(sender.send(true)).expect("Ctrl+C signal send");
        })
            .expect("Error setting Ctrl+C handler");
    }

    tokio::select! {
        Err(e) = watcher_handler => {
            if e.is_panic() { log::error!("The one of watcher actors unexpectedly panic:{}", e) }
            log::error!("Watchers actors aren't supposed to finish any of their execution")
        },
        _ = async { stop_signal_receiver.next().await } => {
            log::warn!("Stop signal received, shutting down");
        }
    };

    Ok(())
}
