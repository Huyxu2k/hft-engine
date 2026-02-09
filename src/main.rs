mod types;
mod execute;
mod risk;
mod trader;
mod engine;
mod core;


fn main() {
    // 1. Khởi tạo các kênh giao tiếp (Channels)
    let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<OrderCommand>();
    let (event_tx, event_rx) = crossbeam_channel::unbounded::<EngineEvent>();

    // 2. Chạy Core Engine (Luồng cực nhanh)
    // Truyền event_tx vào để Core có thể bắn Trade ra ngoài
    thread::spawn(move || {
        let mut book = OrderBook::new();
        while let Ok(cmd) = cmd_rx.recv() {
            // ... Logic Matching Engine cũ ...
            let trades = book.execute_match(...);
            if !trades.is_empty() {
                event_tx.send(EngineEvent::Trades(trades)).unwrap();
            }
        }
    });

    // 3. Chạy Execution Manager (Luồng xử lý I/O mạng chậm)
    start_execution_manager(
        event_rx, 
        "YOUR_API_KEY".to_string(), 
        "YOUR_SECRET".to_string(),
        symbol_config
    );

    // 4. Chạy Binance Gateway (Luồng nhận dữ liệu từ WebSocket)
    // run_binance_gateway(cmd_tx);
}



// 5. Những lưu ý "sống còn" cho phần này:
// Taker vs Maker: Trong luồng Trades, bạn cần phân biệt được Trade nào là của chính bạn sinh ra. Thường thì bạn sẽ kiểm tra maker_id hoặc taker_id có khớp với ID lệnh bạn đã đặt hay không. Bạn không muốn gửi lệnh lên Binance mỗi khi có hai người khác khớp nhau trên bảng điện!

// Rate Limit: Binance có giới hạn số lệnh mỗi giây. Execution Manager nên có một bộ đếm để không gửi quá nhanh gây khóa API.

// Error Handling: Nếu account.limit_buy trả về lỗi (ví dụ: số dư không đủ), bạn cần một cơ chế để đồng bộ ngược lại với Core nhằm hủy lệnh ảo đang treo trong OrderBook nội bộ.