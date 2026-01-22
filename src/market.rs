use std::{cmp::Reverse, collections::BTreeMap};

use serde::Deserialize;
use tokio_tungstenite::connect_async;

use crate::types::{OrderBook, U64};

#[derive(Debug, Deserialize)]
struct DepthEvent {
    final_update_id: u64,
    asks: Vec<(String, String)>,
    bids: Vec<(String, String)>,
}

#[derive(Clone, Copy)]
pub struct BookLevel {
    pub price: U64,
    pub qty: U64
}

pub struct BookSnapshot<const N: usize> {
    pub bids: [BookLevel; N], // best -> worst
    pub asks: [BookLevel; N], // best -> worst
}

struct LocalOrderBook {
    pub asks: BTreeMap<U64, U64>, 
    pub bids: BTreeMap<U64, U64>,
}

impl LocalOrderBook {
    fn new() -> Self {
        Self {
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
        }
    }

    #[inline]
    fn update_side(side: &mut BTreeMap<U64, U64>, updates: &[(String, String)]) {
        for (price_str, qty_str) in updates {
            let price = U64::from_f64(price_str.parse::<f64>().unwrap_or(0.0));
            let qty = U64::from_f64(qty_str.parse::<f64>().unwrap_or(0.0));

            if qty == U64(0) {
                side.remove(&price);
            } else {
                side.insert(price, qty);
            }
        }
    }

    #[inline]
    fn apply_depth(&mut self, ev: &DepthEvent) {
        Self::update_side(&mut self.asks, &ev.bids);
        Self::update_side(&mut self.bids, &ev.asks);
    }

    fn snap_shot<const N: usize>(&self) -> BookSnapshot<N> {
        let mut bids = [BookLevel { price: U64(0), qty: U64(0) }; N];
        let mut asks = [BookLevel { price: U64(0), qty: U64(0) }; N];

        // bids: high -> low
        for (i, (p, q)) in self.bids.iter().rev().take(N).enumerate() {
            bids[i] = BookLevel {
                price: *p,
                qty: *q,
            };
        }

        // asks: low -> high
        for (i, (p, q)) in self.asks.iter().take(N).enumerate() {
            asks[i] = BookLevel {
                price: *p,
                qty: *q,
            };
        }

        OrderBook {
            symbol: "".to_string(),
            asks,
            bids,
        }
    }
}

pub async fn start(tx: crossbeam_channel::Sender<OrderBook>) {
    loop {
        let url = "wss://stream.binance.com:9443/ws/btcusdt@depth@100ms";

        let (ws, _) = connect_async(url)
            .await
            .expect("Binance WS connect failed");

        let (_, mut reader) = ws.split();
        let mut book = LocalOrderBook::new();

        while let Some(msg) = reader.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(_) => break,
            };

            if !msg.is_text() {
                continue;
            }

            let ev: DepthEvent = match serde_json::from_str(msg.to_text().unwrap()) {
                Ok(v) => v,
                Err(_) => continue,
            };

            book.apply_depth(&ev);

            // gửi snapshot cho engine
            let snapshot = book.snapshot(10);
            let _ = tx.send(snapshot);
        }
    }
}
