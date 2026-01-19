use std::cmp::Reverse;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::BTreeMap, u64};

use hashbrown::HashMap;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;

use crate::utils::ExchangeConfig;

// Binance API response structures

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct BinancePrice {
    pub symbol: String,
    pub price: String
}

#[derive(Debug, Clone)]
pub struct BinanceTicker {
    pub symbol: String,
    pub price: String,
    pub volume: String
}

#[derive(Debug, Clone)]
pub struct BinanceOrderBook {
    pub last_update_id: u64,
    pub asks: BTreeMap<u64, u32>,
    pub bids: BTreeMap<Reverse<u64>, u32>
}

pub struct BinanceAPI {
    client: Client,
    config: ExchangeConfig
}

impl BinanceAPI {
    pub fn new (config: ExchangeConfig) -> Self {
        Self { client: Client::new(), config }
    }

    fn generate_signature(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
                        .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn get_timestamp(&self) -> u64 {
        SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
    }

    
}