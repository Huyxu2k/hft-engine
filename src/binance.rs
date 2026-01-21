use std::cmp::Reverse;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::BTreeMap, u64};

use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;

use crate::types::U64;
use crate::utils::{ExchangeConfig, OrderBook, Price};

// Binance API response structures

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize)]
pub struct BinancePrice {
    pub symbol: String,
    pub price: String
}

#[derive(Debug, Deserialize)]
pub struct BinanceTicker {
    pub symbol: String,
    pub price: String,
    pub volume: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceOrderBook {
    pub last_update_id: u64,
    pub asks: Vec<[String; 2]>,
    pub bids: Vec<[String; 2]>,
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
            price: U64::from_f64(price),
            timestamp: self.get_timestamp() / 1000,
            volume: U64::from_f64(volume),
        })
    }
    
    async fn get_24hr_volume(&self, symbol: &str) -> Result<f64, String> {
        let url = format!("{}/api/v3/ticker/24hr", self.config.base_url);

        let res = self.client
                .get(&url)
                .query(&[("symbol", symbol)])
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

        let ticker = res.json::<BinanceTicker>()
                            .await
                            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let volume = ticker
            .volume
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse volume: {}", e))?;

        Ok(volume)
    }

    pub async fn get_orderbook(&self, symbol: &str) -> Result<OrderBook, String> {
        let url = format!("{}/api/v3/depth", self.config.base_url);

        let res = self.client
                    .get(&url)
                    .query(&[("symbol", symbol), ("limit", "10")])
                    .send()
                    .await
                    .map_err(|e| format!("Request failed: {}", e))?;

        let binance_orderbook = res.json::<BinanceOrderBook>()
                        .await
                        .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let mut asks = Vec::<(f64, f64)>::new();

        for ask in binance_orderbook.asks {
            let price = ask[0]
                .parse::<f64>()
                .map_err(|e| format!("Failed to parse ask price: {}", e))?;
            let quantity = ask[1]
                .parse::<f64>()
                .map_err(|e| format!("Failed to parse ask quantity: {}", e))?;
            asks.push((price, quantity));
        }

        // TODO: convert asks/bids into OrderBook and return
        Err("get_orderbook not implemented".to_string())
    }
}