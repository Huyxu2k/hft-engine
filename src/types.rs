

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

