use std::{cmp::Reverse, collections::BTreeMap};
use hashbrown::HashMap;


// API Configuration
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
    pub testnet: bool
}

#[derive(Debug, Clone)]
pub struct Price {
    pub symbol: String,
    pub price: u64,
    pub timestamp: u64,
    pub volume: u64
}

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    // Giá bán
    pub asks: BTreeMap<u64, u32>,
    // Giá mua
    pub bids: BTreeMap<Reverse<u64>, u32>,
    // Lưu trữ chi tiết lệnh
    pub orders: HashMap<u64, Order>,
    // ID cho các giao dịch
    pub trade_counter: u64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Debug)]
pub enum OrderType {
    // Ưu tiên thực hiện ngay theo giá tốt nhất hiện có, có rủi ro trượt giá
    Market,
    // Cho phép đặt giá cụ thể chỉ đảm bảo khớp lệnh khi thị trường đạt tới giá đó
    Limit
}

#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub price: u64,
    pub qty: u64,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub timestamp: u64
}

#[derive(Debug)]
pub struct Position {
    pub symbol: String,
    pub qty: u64,
    // giá trung bình
    pub avr_price: u64,
    // lợi nhuận / thua lỗ chưa thực hiện
    pub unrealized_pnl: u64
}

#[derive(Debug)]
pub struct TradingSignal {
    pub symbol: String,
    pub action: OrderSide,
    pub confidence: u64,
    pub target_price: u64,
    pub qty: u64
}

#[derive(Debug)]
pub struct RiskParams {
    pub max_position_size: u64,
    pub max_loss_per_trade: u64,
    pub max_daily_loss: u64,
    pub stop_loss_pct: u64,
    pub take_profit_pct: u64
}