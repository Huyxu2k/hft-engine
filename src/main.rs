mod core;
mod engine;
mod execute;
mod risk;
mod trader;
mod types;

use crate::engine::{start_core_engine, start_execution_manager};
use crate::execute::run_binance_gateway;
use crate::types::{EngineEvent, OrderCommand, OrderID, U64};
use crossbeam_channel::{Sender, unbounded};
use std::env;
use std::thread;
use std::time::Duration;

fn submit_order(
    cmd_tx: &Sender<OrderCommand>,
    event_tx: &Sender<EngineEvent>,
    id: OrderID,
    price: f64,
    shares: f64,
    is_buy: bool,
) {
    let (resp_tx, resp_rx) = unbounded();

    cmd_tx
        .send(OrderCommand::Add {
            id,
            price: U64::from_f64(price),
            shares: U64::from_f64(shares),
            is_buy,
            resp: resp_tx,
        })
        .expect("failed to send order command to core engine");

    let trades = resp_rx
        .recv_timeout(Duration::from_millis(200))
        .expect("core engine did not respond in time");

    if !trades.is_empty() {
        event_tx
            .send(EngineEvent::Trades(trades))
            .expect("failed to publish trades to execution pipeline");
    }
}

#[tokio::main]
async fn main() {
    // 1) Core matching engine
    let cmd_tx = start_core_engine();

    // 2) Channel đưa event khớp lệnh từ core -> execution manager
    let (event_tx, event_rx) = unbounded::<EngineEvent>();

    // 3) Execution manager (gửi lệnh ra sàn) nếu có API key, ngược lại log local
    let api_key = env::var("BINANCE_API_KEY").ok();
    let secret_key = env::var("BINANCE_SECRET_KEY").ok();

    match (api_key, secret_key) {
        (Some(key), Some(secret)) => {
            start_execution_manager(event_rx, key, secret);
            println!("Execution manager started with Binance account");
        }
        _ => {
            thread::spawn(move || {
                while let Ok(event) = event_rx.recv() {
                    if let EngineEvent::Trades(trades) = event {
                        println!("[Local Execution] matched trades: {:?}", trades);
                    }
                }
            });
            println!("BINANCE_API_KEY/BINANCE_SECRET_KEY not set, using local execution logger");
        }
    }

    // 4) Optional market-data gateway (Binance websocket -> core)
    if env::var("HFT_ENABLE_BINANCE_GATEWAY").ok().as_deref() == Some("1") {
        let gateway_tx = cmd_tx.clone();
        tokio::spawn(async move {
            run_binance_gateway(gateway_tx).await;
        });
        println!("Binance gateway enabled");
    }

    // 5) Demo chiến lược: gửi 2 lệnh để tạo luồng đầy đủ trader -> core -> execution
    submit_order(&cmd_tx, &event_tx, 1, 50000.0, 0.01, false); // maker sell
    submit_order(&cmd_tx, &event_tx, 2, 50000.0, 0.01, true); // taker buy => tạo trade

    // Giữ process để các worker threads/gateway tiếp tục chạy
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
