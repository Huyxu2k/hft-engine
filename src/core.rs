// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25

type OrderId = u64;
type Price = i32;
pub struct Order {
    id: OrderId,
    shares: u32,
    price: Price,
    // Index thay cho con trỏ để quản lý bộ nhớ nhanh hơn
    prev: Option<usize>,
    next: Option<usize>,
    limit_parent: Price
}

pub struct Limit {
    price: Price,
    total_volume: u64,
    head: Option<usize>, // Index của Order đầu tiên
    tail: Option<usize>  // Index của Order cuối cùng
}

// triển khai Order Book

use std::collections::{BTreeMap, HashMap};

pub struct OrderBook {
    
    // TODO Option<Order> Sử dụng Option để đánh dấu ô trống
    orders_arena: Vec<Order>,               // Arena lưu trữ tất cả các lệnh để tránh cấp phát nhỏ lẻ
    // TODO Lưu trữ các index có thể tái sử dụng
    //free_list: Vec<usize>,
    id_map: HashMap<OrderId, usize>,        // Ánh xạ OrderId sang vị trí trong Arena
    buy_limits: BTreeMap<Price, Limit>,     //  Các mức giá (Limit) mua
    sell_limits: BTreeMap<Price, Limit>     //  Các mức giá (Limit) bán
}

impl OrderBook {
    pub fn new() -> Self {
        Self { 
            orders_arena: Vec::with_capacity(100_000), 
            id_map: HashMap::with_capacity(100_000), 
            buy_limits: BTreeMap::new(), 
            sell_limits: BTreeMap::new() 
        }
    }

    /// Thêm lệnh mới - O(log M)
    pub fn add_order(&mut self, id: OrderId, price: Price, shares: u32, is_buy: bool) {
        let limits = if is_buy { &mut self.buy_limits } else { &mut self.sell_limits };
        
        let limit = limits.entry(price).or_insert(Limit {
            price,
            total_volume: 0,
            head: None,
            tail: None,
        });

        // Tạo order mới
        let new_idx = self.orders_arena.len();
        let mut order = Order {
            id,
            shares,
            price,
            prev: limit.tail, // Trỏ về order cuối cũ
            next: None,
            limit_parent: price
        };

        // Cập nhật liên kết đôi
        if let Some(old_tail_idx) = limit.tail {
            self.orders_arena[old_tail_idx].next = Some(new_idx);
        } else {
            limit.head = Some(new_idx);
        }

        limit.tail = Some(new_idx);
        limit.total_volume += shares as u64;
    }

    /// Hủy lệnh bất kỳ - O(1) trung bình
    pub fn cancel_order(&mut self, id: OrderId) {
        if let Some(&idx) = self.id_map.get(&id) {
            let price = self.orders_arena[idx].limit_parent;
            let shares = self.orders_arena[idx].shares;
            let prev_idx = self.orders_arena[idx].prev;
            let next_idx = self.orders_arena[idx].next;
            
            // Tìm Limit tương ứng (giả sử Buy để minh họa)
            if let Some(limit) = self.buy_limits.get_mut(&price) {
                // Ngắt kết nối Link List
                if let Some(p) = prev_idx { self.orders_arena[p].next = next_idx; }
                else { limit.head = next_idx; }

                if let Some(n) = next_idx { self.orders_arena[n].prev = prev_idx; }
                else { limit.tail = prev_idx; }

                limit.total_volume -= shares as u64;
            }
            self.id_map.remove(&id);
        }
    }

    pub fn execute_match(&mut self, taker_id: OrderId, price: Price, mut shares_left: u32, is_buy: bool) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Nếu là lệnh Buy, ta tìm trong sell_limits (giá thấp nhất trước)
        // Nếu là lệnh Sell, ta tìm trong buy_limits (giá cao nhất trước)
        let side_limits = if is_buy { &mut self.sell_limits } else { &mut self.buy_limits };

        while shares_left > 0 {
            // Lấy mức giá tốt nhất hiện tại
            let best_price = if is_buy {
                side_limits.keys().next().cloned() // Min price cho Sell side
            } else {
                side_limits.keys().next_back().cloned() // Max price cho Buy side
            };

            match best_price {
                Some(p) if (is_buy && p <= price) || (!is_buy && p >= price) => {
                    let limit = side_limits.get_mut(&p).unwrap();
                    
                    while let Some(maker_idx) = limit.head {
                        let maker_order = self.orders_arena[maker_idx].as_mut().unwrap();
                        let traded_shares = std::cmp::min(shares_left, maker_order.shares);

                        // Tạo Trade
                        trades.push(Trade {
                            maker_id: maker_order.id,
                            taker_id,
                            price: p,
                            shares: traded_shares,
                        });

                        // Cập nhật khối lượng
                        maker_order.shares -= traded_shares;
                        shares_left -= traded_shares;
                        limit.total_volume -= traded_shares as u64;

                        if maker_order.shares == 0 {
                            // Lệnh maker đã khớp hết, xóa khỏi list
                            let next_maker = maker_order.next;
                            self.id_map.remove(&maker_order.id);
                            let old_idx = maker_idx;
                            
                            limit.head = next_maker;
                            if let Some(next_idx) = next_maker {
                                self.orders_arena[next_idx].as_mut().unwrap().prev = None;
                            } else {
                                limit.tail = None;
                            }
                            
                            // Trả về pool
                            //self.orders_arena[old_idx] = None;
                            //self.free_list.push(old_idx);
                        }

                        if shares_left == 0 { break; }
                    }

                    // Nếu mức giá này không còn lệnh nào, xóa luôn Limit
                    if limit.head.is_none() {
                        side_limits.remove(&p);
                    }
                }
                _ => break, // Không còn giá khớp hoặc hết lệnh đối ứng
            }
        }
        trades
    }
}

// 
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;

// Định nghĩa các loại tin nhắn mà Core có thể xử lý
enum OrderCommand {
    Add { id: OrderId, price: Price, shares: u32, is_buy: bool , resp: Sender<Vec<Trade>>},
    Cancel { id: OrderId },
}

fn start_core_engine() -> Sender<OrderCommand> {
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


// Matching Engine
#[derive(Debug)]
pub struct Trade {
    pub maker_id: OrderId,
    pub taker_id: OrderId,
    pub price: Price,
    pub shares: u32,
}
