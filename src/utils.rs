use std::{cmp::Reverse, collections::BTreeMap};
use hashbrown::HashMap;

use crate::types::U64;


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
    pub price: U64,
    pub timestamp: u64,
    pub volume: U64
}

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    // Giá bán
    pub asks: BTreeMap<U64, u32>,
    // Giá mua
    pub bids: BTreeMap<Reverse<U64>, u32>,
    // Lưu trữ chi tiết lệnh
    pub orders: HashMap<U64, Order>,
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
    pub price: U64,
    pub qty: U64,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub timestamp: u64
}

#[derive(Debug)]
pub struct Position {
    pub symbol: String,
    pub qty: U64,
    // giá trung bình
    pub avr_price: U64,
    // lợi nhuận / thua lỗ chưa thực hiện
    pub unrealized_pnl: U64
}

#[derive(Debug)]
pub struct TradingSignal {
    pub symbol: String,
    pub action: OrderSide,
    pub confidence: U64,
    pub target_price: U64,
    pub qty: U64
}

#[derive(Debug)]
pub struct RiskParams {
    pub max_position_size: U64,
    pub max_loss_per_trade: U64,
    pub max_daily_loss: U64,
    pub stop_loss_pct: U64,
    pub take_profit_pct: U64
}