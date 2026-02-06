use crossbeam_channel::bounded;
use crate::{core::OrderBook, types::OrderCommand};

// 
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
                    let matched_shares: u32 = trades.iter().map(|t| t.shares).sum();
                    shares -= matched_shares;

                    // 2. Nếu còn dư thì mới thêm vào Book (Passive Order)
                    if shares > 0 {
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