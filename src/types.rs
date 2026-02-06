use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::fmt;

use crossbeam_channel::Sender;

use hashbrown::HashMap;

// Const
pub type OrderID = u64;
pub type Price = U64;

pub enum Side {
    Buy,
    Sell,
    Hold,
}


#[derive(Debug)]
pub struct Trade {
    pub maker_id: OrderID,
    pub taker_id: OrderID,
    pub price: Price,
    pub shares: U64,
}


// Định nghĩa các loại tin nhắn mà Core có thể xử lý
pub enum OrderCommand {
    Add { id: OrderID, price: Price, shares: u32, is_buy: bool , resp: Sender<Vec<Trade>>},
    Cancel { id: Price },
}


// Fixed-Point
pub const SCALE_FACTOR: u64 = 100_000_000;
const SCALE_FACTOR_F64: f64 = 100_000_000.0;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U64(pub u64);

impl U64 {
    /// Fixed Point f64 to u64
    pub fn from_f64(val: f64) -> Self {
        Self((val * SCALE_FACTOR_F64 + 0.5) as u64)
    }

    /// Origin value
    pub fn org_val(self) -> f64 {
        self.0 as f64 / SCALE_FACTOR_F64
    }

    // U64 value
    pub fn from_val(val: u64) -> Self {
        Self(val)
    }

    /// u64 value
    pub fn val(self) -> u64 {
        self.0
    }

    pub fn zero() -> U64{
        U64(0)
    }
}

impl Add for U64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for U64 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for U64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for U64 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
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
        write!(f, "{:.8}", self.org_val())
    }
}