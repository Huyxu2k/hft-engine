use crate::{core::OrderBook, types::{EngineEvent, OrderCommand, U64}};
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;

pub fn start_core_engine() -> Sender<OrderCommand> {
    let (tx, rx): (Sender<OrderCommand>, Receiver<OrderCommand>) = unbounded();

    thread::spawn(move || {
        let mut book = OrderBook::new();

        // Vòng lặp vô tận xử lý tin nhắn với độ trễ thấp nhất
        while let Ok(cmd) = rx.recv() {
            match cmd {
                OrderCommand::Add { id, price, mut shares, is_buy , resp} => {
                    // 1. Chạy Matching Engine trước
                    let trades = book.execute_match(id, price, shares, is_buy);

                    // Tính khối lượng đã khớp để trừ đi
                    let matched_shares: U64 = trades.iter().map(|t| t.shares).sum();
                    shares -= matched_shares;

                    // 2. Nếu còn dư thì mới thêm vào Book (Passive Order)
                    if shares > U64::zero() {
                        book.add_order(id, price, shares, is_buy);
                    }

                    // 3. Gửi thông báo giao dịch về luồng gửi
                    let _ = resp.send(trades);
                }
                OrderCommand::Cancel { id } => {
                    book.cancel_order(id);
                }
            }
            // Sau mỗi lệnh có thể thực hiện Match Engine tại đây
        }
    });
    // Trả về Sender để các luồng khác gửi lệnh vào
    tx
}

use binance::account::Account;
use binance::api::Binance;

pub fn start_execution_manager(
    event_rx: Receiver<EngineEvent>, 
    api_key: String, 
    secret_key: String,
    //config: SymbolConfig // Chứa các multiplier đã nói ở trên
) {
    let account: Account = Binance::new(Some(api_key), Some(secret_key));

    thread::spawn(move || {
        while let Ok(event) = event_rx.recv() {
            if let EngineEvent::Trades(trades) = event {
                for trade in trades {
                    // 1. Chuyển đổi ngược từ U64 về f64/String cho Binance
                    let price_f64 = trade.price.org_val();
                    let qty_f64 = trade.shares.org_val();

                    // 2. Gửi lệnh (Bạn có thể thêm logic kiểm tra maker/taker ID 
                    // để chỉ gửi lệnh của chính bạn, thay vì lệnh của market data)
                    match account.limit_buy("BTCUSDT", qty_f64, price_f64) {
                        Ok(answer) => println!("Binance Order Executed: {:?}", answer),
                        Err(e) => eprintln!("Execution Error: {:?}", e),
                    }
                }
            }
        }
    });
}

// #[derive(Debug, Clone)]
// pub struct SymbolConfig {
//     pub symbol: String,
    
//     // Hệ số nhân để biến String/Float thành u64
//     pub price_multiplier: u64, 
//     pub qty_multiplier: u64,
    
//     // Các giới hạn an toàn từ sàn
//     pub min_notional: u128, // Giá trị lệnh tối thiểu (Price * Qty)
//     pub tick_size: f64,     // Để dùng khi cần format ngược lại String
// }

// impl SymbolConfig {
//     /// Hàm tiện ích để chuyển đổi giá từ u64 (nội bộ) sang f64 (để gửi lên API sàn)
//     pub fn to_f64_price(&self, internal_price: u64) -> f64 {
//         internal_price as f64 / self.price_multiplier as f64
//     }

//     /// Hàm tiện ích để chuyển đổi khối lượng từ u64 sang f64
//     pub fn to_f64_qty(&self, internal_qty: u64) -> f64 {
//         internal_qty as f64 / self.qty_multiplier as f64
//     }
// }


// Giả mã logic lấy cấu hình từ Binance API
// async fn fetch_symbol_config(symbol: &str) -> SymbolConfig {
//     // Gọi GET https://api.binance.com/api/v3/exchangeInfo?symbol=BTCUSDT
//     // Giả sử nhận được tickSize = "0.01" và stepSize = "0.00001"
    
//     SymbolConfig {
//         symbol: symbol.to_string(),
//         price_multiplier: 100,      // Tính toán từ tickSize
//         qty_multiplier: 100000,     // Tính toán từ stepSize
//         min_notional: 10 * 100 * 100000, // Thường là 10 USDT
//         tick_size: 0.01,
//     }
// }