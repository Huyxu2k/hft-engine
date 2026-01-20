use crate::types::{Execution, Order, Side};
use hashbrown::HashMap;
use std::cmp::Reverse;
use std::{collections::BTreeMap, u64};




#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub price: u64,
    pub qty: u32,
    pub side: Side
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub trade_id: u64,
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: u64,
    pub qty: u32
}


pub struct OrderBook {
    pub symbol: String,
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
    // Lưu trữ chi tiết lệnh
    pub orders: HashMap<u64, Order>,
    // ID cho các giao dịch
    pub trade_counter: u64,
    pub timestamp: u64,
}

pub struct OrderBook {
    pub symbol: String,
    pub asks: BTreeMap<u64, u32>,
    pub bids: BTreeMap<Reverse<u64>, u32>,
    // Lưu trữ chi tiết lệnh
    pub orders: HashMap<u64, Order>,
    // ID cho các giao dịch
    pub trade_counter: u64,
    pub timestamp: u64,
}

pub struct OrderBook {
    // asks: Bán thấp -> cao (key: u64)
    asks: BTreeMap<u64, u32>,
    // bids: Mua cao -> thấp (key: Reverse<u64> để BTreeMap tự sắp xếp giảm dần)
    bids: BTreeMap<Reverse<u64>, u32>,

    // Lưu trữ chi tiết lệnh
    orders: HashMap<u64, Order>,

    // ID cho các giao dịch
    trade_counter: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            orders: HashMap::new(),
            trade_counter: 0,
        }
    }

    pub fn process_order(&mut self, incoming_order: Order) -> Vec<Execution> {
        let mut executions: Vec<Execution> = Vec::new();
        let mut remaining_qty = incoming_order.qty;
        
        // Cố gắng khớp lệnh trước khi thêm phần còn lại vào sổ lệnh
        if incoming_order.side == Side::Buy {
            // Check nếu có lệnh bán (asks) có giá <= giá mua của lệnh mới
            while remaining_qty > 0 && !self.asks.is_empty() {
                let (&best_ask_price, &best_ask_qty) = {
                    let first_entry = self.asks.iter().next().unwrap();
                    (first_entry.0, first_entry.1)
                };
                
                if best_ask_price > incoming_order.price { break; } // Không khớp được nữa

                // Logic khớp lệnh: lấy min(số lượng còn lại, số lượng tốt nhất)
                let matched_qty = std::cmp::min(remaining_qty, best_ask_qty);
                remaining_qty -= matched_qty;

                // Cập nhật sổ lệnh Ask và chi tiết lệnh Maker
                self.update_level(&mut self.asks, best_ask_price, best_ask_qty - matched_qty);
                self.update_maker_order(best_ask_price, Side::Sell, matched_qty);

                // Ghi nhận giao dịch
                executions.push(Execution {
                    trade_id: self.trade_counter,
                    maker_order_id: self.get_order_id_at_price(best_ask_price, Side::Sell), // Cần hàm phụ trợ
                    taker_order_id: incoming_order.id,
                    price: best_ask_price,
                    qty: matched_qty,
                });
                self.trade_counter += 1;
            }
        } else {
            // Tương tự cho lệnh Sell, khớp với Bids có giá >= giá bán của lệnh mới
            while remaining_qty > 0 && !self.bids.is_empty() {
                let (&Reverse(best_bid_price), &best_bid_qty) = self.bids.iter().next().unwrap();

                if best_bid_price < incoming_order.price { break; } // Không khớp được nữa

                let matched_qty = std::cmp::min(remaining_qty, best_bid_qty);
                remaining_qty -= matched_qty;

                self.update_level(&mut self.bids, Reverse(best_bid_price), best_bid_qty - matched_qty);
                self.update_maker_order(best_bid_price, Side::Buy, matched_qty);
                
                // Ghi nhận giao dịch (cần hàm phụ trợ để lấy ID)
                 executions.push(Execution {
                    trade_id: self.trade_counter,
                    maker_order_id: self.get_order_id_at_price(best_bid_price, Side::Buy),
                    taker_order_id: incoming_order.id,
                    price: best_bid_price,
                    qty: matched_qty,
                });
                self.trade_counter += 1;
            }
        }

        // Nếu còn lại số lượng, thêm phần còn lại vào sổ lệnh (trở thành Maker Order)
        if remaining_qty > 0 {
            let final_order = Order {
                qty: remaining_qty,
                ..incoming_order.clone()
            };
            self.insert_remaining(final_order);
        }

        executions
    }

    // Hàm phụ trợ để lấy ID lệnh tại mức giá (cần cấu trúc sổ lệnh phức tạp hơn 1 chút nếu muốn lấy chính xác ID của lệnh đầu tiên, ở đây ta giả định đơn giản)
    fn get_order_id_at_price(&self, price: u64, side: Side) -> u64 {
        // Trong hệ thống thực tế, mỗi mức giá sẽ là một VecDeque<Order> (hàng đợi FIFO)
        // Để đơn giản, ta chỉ trả về ID của lệnh đầu tiên tìm thấy trong HashMap có giá tương ứng.
        // Đây là điểm yếu trong ví dụ đơn giản này.
        self.orders.iter()
           .find(|&(_, order)| order.price == price && order.side == side)
           .map(|(&id, _)| id)
           .unwrap_or(0)
    }
    fn update_level<K: Ord>(&mut self, book: &mut BTreeMap<K, u32>, key: K, new_qty: u32) {
        if new_qty == 0 {
            book.remove(&key);
        } else {
            *book.get_mut(&key).unwrap() = new_qty;
        }
    }

    fn update_maker_order(&mut self, price: u64, side: Side, matched_qty: u32) {
        // Cần vòng lặp để tìm lệnh maker chính xác đã khớp.
        // Trong hệ thống HFT thực tế:
        // bids/asks lưu trữ VecDeque<Order> chứ không chỉ tổng volume.
        // Khi khớp, ta pop_front() các lệnh cũ ra khỏi VecDeque.
        // Ví dụ đơn giản này chỉ cập nhật tổng volume.
    }

    fn insert_remaining(&mut self, order: Order) {
        let price = order.price;
        let qty = order.qty;
        match order.side {
            Side::Buy => {
                *self.bids.entry(Reverse(price)).or_insert(0) += qty;
            }
            Side::Sell => {
                *self.asks.entry(price).or_insert(0) += qty;
            }
        }
        self.orders.insert(order.id, order);
    }
}


// Cấu trúc dữ liệu tối ưu: Sử dụng &str để tránh copy dữ liệu (Zero-copy)
#[derive(Deserialize, Debug)]
pub struct BinanceTicker<'a> {
    #[serde(rename = "s")]
    pub symbol: &'a str,
    #[serde(rename = "a")]
    pub best_ask: &'a str,
    #[serde(rename = "A")]
    pub ask_qty: &'a str,
    #[serde(rename = "b")]
    pub best_bid: &'a str,
    #[serde(rename = "B")]
    pub bid_qty: &'a str,
}

pub struct MarketDataHandler {
    symbol: String,
    tx: mpsc::Sender<String>, // Gửi dữ liệu thô sang Strategy Engine
}

impl MarketDataHandler {
    pub fn new(symbol: &str, tx: mpsc::Sender<String>) -> Self {
        Self {
            symbol: symbol.to_lowercase(),
            tx,
        }
    }

    pub async fn start_loop(&self) {
        // Binance Stream URL cho Aggregate Trade hoặc Book Ticker
        let url = format!("wss://stream.binance.com:9443/ws/{}@bookTicker", self.symbol);
        let url = Url::parse(&url).unwrap();

        loop {
            println!("Connecting to Binance WebSocket: {}", url);
            
            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    let (_, mut read) = ws_stream.split();
                    
                    while let Some(message) = read.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                // Gửi thẳng dữ liệu thô sang Engine để xử lý ở thread khác
                                // Tránh block việc nhận data từ socket
                                if let Err(e) = self.tx.send(text).await {
                                    eprintln!("Channel error: {}", e);
                                    break;
                                }
                            }
                            Ok(Message::Ping(_)) => continue,
                            Err(e) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            _ => (),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed: {}. Retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}


#[tokio::main]
async fn main() {
    // Khởi tạo channel với buffer size để tránh treo hệ thống (Backpressure)
    let (tx, mut rx) = mpsc::channel::<String>(1000);

    // Khởi chạy Market Data Ingestion trong một task riêng
    let handler = MarketDataHandler::new("btcusdt", tx);
    tokio::spawn(async move {
        handler.start_loop().await;
    });

    // Logic xử lý dữ liệu (Strategy Engine)
    println!("Strategy Engine started...");
    while let Some(raw_data) = rx.recv().await {
        // Parse dữ liệu cực nhanh với Zero-copy
        let parse_result: Result<BinanceTicker, _> = serde_json::from_str(&raw_data);
        
        match parse_result {
            Ok(ticker) => {
                // Đây là nơi logic HFT của bạn bắt đầu
                // Ví dụ: So sánh giá Binance vs Bybit
                println!(
                    "Symbol: {} | Bid: {} | Ask: {}", 
                    ticker.symbol, ticker.best_bid, ticker.best_ask
                );
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}