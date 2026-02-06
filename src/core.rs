// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
use crate::types::OrderID;
use crate::types::Price;
use crate::types::Trade;
use crate::types::U64;

pub struct Order {
    id: OrderID,
    shares: U64,
    price: Price,
    // Index thay cho con trỏ để quản lý bộ nhớ nhanh hơn
    prev: Option<usize>,
    next: Option<usize>,
    limit_parent: Price
}

pub struct Limit {
    price: Price,
    total_volume: U64,
    head: Option<usize>, // Index của Order đầu tiên
    tail: Option<usize>  // Index của Order cuối cùng
}

// triển khai Order Book

use std::collections::{BTreeMap, HashMap};

// Trait
pub trait  IOrderBook {
    fn execute_match(&self);
}

pub struct OrderBook {
    // TODO Option<Order> Sử dụng Option để đánh dấu ô trống
    orders_arena: Vec<Order>,               // Arena lưu trữ tất cả các lệnh để tránh cấp phát nhỏ lẻ
    // TODO Lưu trữ các index có thể tái sử dụng
    //free_list: Vec<usize>,
    id_map: HashMap<OrderID, usize>,        // Ánh xạ OrderID sang vị trí trong Arena
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
    pub fn add_order(&mut self, id: OrderID, price: Price, shares: U64, is_buy: bool) {
        let limits = if is_buy { &mut self.buy_limits } else { &mut self.sell_limits };
        
        let limit = limits.entry(price).or_insert(Limit {
            price,
            total_volume: U64::zero(),
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
        limit.total_volume += shares as U64;
    }

    /// Hủy lệnh bất kỳ - O(1) trung bình
    pub fn cancel_order(&mut self, id: OrderID) {
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

                limit.total_volume -= shares as U64;
            }
            self.id_map.remove(&id);
        }
    }

    pub fn execute_match(&mut self, taker_id: OrderID, price: Price, mut shares_left: U64, is_buy: bool) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Nếu là lệnh Buy, ta tìm trong sell_limits (giá thấp nhất trước)
        // Nếu là lệnh Sell, ta tìm trong buy_limits (giá cao nhất trước)
        let side_limits = if is_buy { &mut self.sell_limits } else { &mut self.buy_limits };

        while shares_left > U64::zero() {
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
                        let maker_order = &mut self.orders_arena[maker_idx];
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
                        limit.total_volume -= traded_shares as U64;

                        if maker_order.shares == U64::zero() {
                            // Lệnh maker đã khớp hết, xóa khỏi list
                            let next_maker = maker_order.next;
                            self.id_map.remove(&maker_order.id);
                            let old_idx = maker_idx;
                            
                            limit.head = next_maker;
                            if let Some(next_idx) = next_maker {
                                self.orders_arena[next_idx].prev = None;
                            } else {
                                limit.tail = None;
                            }
                            
                            // Trả về pool
                            //self.orders_arena[old_idx] = None;
                            //self.free_list.push(old_idx);
                        }

                        if shares_left == U64::zero() { break; }
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