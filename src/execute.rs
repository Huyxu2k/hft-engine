use binance::websockets::*;
use binance::api::*;
use std::sync::atomic::AtomicBool;

async fn run_binance_gateway(cmd_tx: Sender<OrderCommand>) {
    let keep_running = AtomicBool::new(true);
    let symbol = "BTCUSDT".to_string();

    // 1. Khởi tạo WebSocket để nhận Diff. Depth Stream (Thay đổi sổ lệnh)
    let mut web_socket = WebSockets::new(move |event: WebsocketEvent| {
        if let WebsocketEvent::OrderBook(depth) = event {
            // Biến đổi dữ liệu Binance thành lệnh cho Core của bạn
            for ask in depth.asks {
                let price = (ask.price * 100.0) as i32; // Chuyển sang fixed-point
                let shares = (ask.qty * 1000.0) as u32;
                
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

// execute
// Trong luồng xử lý Trade
let api_key = Some("YOUR_KEY".into());
let secret_key = Some("YOUR_SECRET".into());
let account: Account = Binance::new(api_key, secret_key);

match account.limit_buy("BTCUSDT", 1.0, 60000.0) {
    Ok(answer) => println!("Order placed: {:?}", answer),
    Err(e) => println!("Error: {:?}", e),
}
