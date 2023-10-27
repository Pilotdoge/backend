use std::env;
#[derive(Debug,Clone)]
pub struct Config {
    pub port: u16,
    pub workers: u16,
    pub tokens_number_per_gas: String,
    pub eligible_min_gas: String,
    pub token_address: String,
    pub token_decimal: u32,
    pub database_url: String,
    pub db_pool_size: u16,
    pub remote_web3_url: String,
    pub sync_start_block: u64,
    pub claim_start: bool,
}

impl Config {
    pub fn from_env() ->Self {
        let port = env::var("SERVER_PORT").unwrap_or_default()
            .parse::<u16>().unwrap_or(8088u16);
        let workers = env::var("WORKERS_NUMBER").unwrap_or_default()
            .parse::<u16>().unwrap_or(2u16);
        let tokens_number_per_gas = env::var("TOKENS_NUMBER_PER_GAS").unwrap_or_default();
        let eligible_min_gas = env::var("ELIGIBLE_MIN_GAS").unwrap_or_default();
        let token_address = env::var("TOKEN_ADDRESS").unwrap_or_default();

        let database_url = env::var("DATABASE_URL").unwrap_or_default();
        let remote_web3_url = env::var("REMOTE_WEB3_URL").unwrap_or_default();
        let db_pool_size = env::var("DB_POOL_SIZE").unwrap_or_default()
            .parse::<u16>().unwrap_or(1u16);
        let sync_start_block = env::var("SYNC_START_BLOCK").unwrap_or_default()
            .parse::<u64>().unwrap_or(0u64);
        let token_decimal = env::var("TOKEN_DECIMAL").unwrap_or_default()
            .parse::<u32>().unwrap_or(0u32);
        let claim_start = env::var("CLAIM_START").unwrap_or_default()
            .parse::<bool>().unwrap_or(false);
        Self {
            port,
            workers,
            tokens_number_per_gas,
            eligible_min_gas,
            token_address,
            token_decimal,
            database_url,
            db_pool_size,
            remote_web3_url,
            sync_start_block,
            claim_start
        }
    }
}