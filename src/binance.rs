use std::cmp::Reverse;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::BTreeMap, u64};

use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;

use crate::utils::{ExchangeConfig, Price};

// Binance API response structures

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize)]
pub struct BinancePrice<'a> {
    pub symbol: &'a str,
    pub price: &'a str
}

#[derive(Debug)]
pub struct BinanceTicker<'a> {
    pub symbol: &'a str,
    pub price: &'a str,
    pub volume: &'a str
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

    pub async fn get_price(&self, symbol: &str) -> Result<Price, String> {
        let url = format!("{}/api/v3/ticker/price", self.config.base_url);

        let res = self.client
                    .get(url)
                    .query(&[("symbol", symbol)])
                    .send()
                    .await
                    .map_err(|e| format!("Request failed: {}", e))?;
         if !res.status().is_success() {
            return Err(format!("API error: {}", res.status()));
        }

        let binance_price = res
            .json::<BinancePrice>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Get volume separately to avoid error across await
        let volume = match self.get_24hr_volume(symbol).await {
            Ok(v) => v,
            Err(_) => 0.0, // Default volume if fetch fails
        };

        let price = binance_price
            .price
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse price: {}", e))?;

        Ok(Price {
            symbol: binance_price.symbol.to_string(),
            price: price as u64,
            timestamp: self.get_timestamp() / 1000,
            volume: volume as u64,
        })
    }
}