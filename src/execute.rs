use binance::websockets::*;
use crossbeam_channel::Sender;
use std::sync::atomic::AtomicBool;

use crate::types::{OrderCommand, U64};

pub async fn run_binance_gateway(cmd_tx: Sender<OrderCommand>) {
    let keep_running = AtomicBool::new(true);
    let symbol = "BTCUSDT".to_string();

    // 1. Khởi tạo WebSocket để nhận Diff. Depth Stream (Thay đổi sổ lệnh)
    let mut web_socket = WebSockets::new(move |event: WebsocketEvent| {
        if let WebsocketEvent::OrderBook(depth) = event {
            // Biến đổi dữ liệu Binance thành lệnh cho Core của bạn
            for ask in depth.asks {
                let price = U64::from_f64(ask.price); 
                let shares = U64::from_f64(ask.qty);
                
                // Gửi lệnh vào Core để cập nhật sổ lệnh nội bộ
                cmd_tx.send(OrderCommand::Add {
                    id: 0, // Dữ liệu sàn không có ID lệnh cá nhân
                    price,
                    shares,
                    is_buy: false,
                    resp: crossbeam_channel::unbounded().0, // Bypass response cho market data
                }).unwrap();
            }
        }
        Ok(())
    });

    web_socket.connect(&format!("{}@depth", symbol.to_lowercase())).unwrap();
    web_socket.event_loop(&keep_running).unwrap();
}
