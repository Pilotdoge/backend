use std::cmp;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;
use anyhow::format_err;
use tokio::task::JoinHandle;
use web3::transports::Http;
use web3::types::{BlockNumber, FilterBuilder, H160, H256, Log};
use web3::Web3;
use crate::config::Config;
use crate::db;
use crate::db::tables::ClaimedAccount;
use crate::watcher::event::ClaimEvent;

#[derive(Clone)]
pub struct ChainWatcher {
    pub config: Config,
    pub web3: Web3<Http>,
    pub db: rbatis::RBatis,
}
impl ChainWatcher {
    pub async fn new(config:Config,db: rbatis::RBatis) -> anyhow::Result<Self> {
        let transport = Http::new(&config.remote_web3_url).unwrap();
        let web3 = Web3::new(transport);
        Ok(Self {
            web3,
            config,
            db,
        })
    }
    async fn sync_claim_events(
        &mut self,
        from: u64,
        to: u64,
    ) -> anyhow::Result<()> {
        let abi_string = r#"[ {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "account",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "claimed_time",
                    "type": "uint256"
                }
            ],
            "name": "Claimed",
            "type": "event"
        }]"#;
        let token_contract = ethabi::Contract::load(abi_string.as_bytes()).unwrap();
        let topic = token_contract
            .event("Claimed")
            .expect("token contract abi error")
            .signature();
        let token_address = H160::from_str(&self.config.token_address.clone()).unwrap();
        let logs: Vec<ClaimEvent> = self.sync_events(from,to, vec![token_address], vec![topic]).await?;
        if !logs.is_empty() {
            let accounts = logs.iter().map(|l| (*l).clone().into())
                .collect::<Vec<ClaimedAccount>>();
            db::save_claimed_accounts(&mut self.db, accounts).await?;
        }
        Ok(())
    }
    async fn sync_events<T>(
        &mut self,
        from: u64,
        to: u64,
        address: Vec<H160>,
        topics: Vec<H256>
    ) -> anyhow::Result<Vec<T>>
        where
            T: TryFrom<Log>,
            T::Error: Debug,
    {
        log::info!("sync events from:{} to:{}",from,to);
        let filter = FilterBuilder::default()
            .address(address)
            .from_block(BlockNumber::Number(from.into()))
            .to_block(BlockNumber::Number(to.into()))
            .topics(Some(topics), None, None, None)
            .build();
        let mut logs = self.web3.eth().logs(filter).await?;
        println!("logs is {:?}",logs);
        let is_possible_to_sort_logs = logs.iter().all(|log| log.log_index.is_some());
        if is_possible_to_sort_logs {
            logs.sort_by_key(|log| {
                log.log_index
                    .expect("all logs log_index should have values")
            });
        } else {
            log::warn!("Some of the log entries does not have log_index, we rely on the provided logs order");
        }


        logs.into_iter()
            .map(|event| {
                T::try_from(event)
                    .map_err(|e| format_err!("Failed to parse event log from ETH: {:?}", e))
            })
            .collect()
    }

    async fn run_sync_events(&mut self) ->anyhow::Result<()> {
        let last_synced_block = db::get_last_sync_block(&self.db,self.config.sync_start_block).await?;
        let chain_block_number = self.web3.eth().block_number().await?.as_u64();
        let sync_step = 1000u64;
        let mut start_block = last_synced_block + 1;
        let mut end_block;
        loop {
            end_block = cmp::min(chain_block_number,start_block + sync_step);
            if start_block > end_block {
                break;
            }
            self.sync_claim_events(start_block,end_block)
                .await.map_err(|e| format_err!("sync_claim_events failed,{:?}",e))?;

            start_block = end_block + 1;
            db::upsert_last_sync_block(
                &mut self.db,
                end_block as i64,
            ).await?;

        }
        Ok(())
    }

    pub async fn run_watcher_server(mut self) {
        let mut tx_poll = tokio::time::interval(Duration::from_secs(120));
        loop {
            tx_poll.tick().await;
            if self.config.claim_start {
                if let Err(e) = self.run_sync_events().await {
                    log::error!("run_sync_pair_events error occurred {:?}", e);
                }
            }

        }
    }
}
pub async fn run_watcher(config: Config, db: rbatis::RBatis) -> JoinHandle<()> {
    log::info!("Starting watcher!");
    let watcher = ChainWatcher::new(config, db).await.unwrap();
    tokio::spawn(watcher.clone().run_watcher_server())
}