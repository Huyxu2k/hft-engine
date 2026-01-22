use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;

use hashbrown::HashMap;


pub struct OrderBook {
    pub symbol: String,
    pub asks: BTreeMap<U64, U64>, // price, qty
    pub bids: BTreeMap<Reverse<U64>, U64>,
    // Lưu trữ chi tiết lệnh
    //pub orders: HashMap<u64, Order>,
}

pub enum Side {
    Buy,
    Sell,
    Hold,
}

pub const SCALE_FACTOR: u64 = 100_000_000;
const SCALE_FACTOR_F64: f64 = 100_000_000.0;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U64(pub u64);

impl U64 {
    pub fn from_f64(val: f64) -> Self {
        Self((val * SCALE_FACTOR_F64 + 0.5) as u64)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE_FACTOR_F64
    }

    pub fn from_raw(val: u64) -> Self {
        Self(val)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl Add for U64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for U64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul for U64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Div for U64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl fmt::Display for U64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.8}", self.to_f64())
    }
}