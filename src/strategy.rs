use crate::types::{OrderBook, Side};

pub fn decide(book: &OrderBook) -> Side {
    let bid: u64 = book.bids.iter().take(5).map(|(_, q)| q).sum();
    let ask: u64 = book.asks.iter().take(5).map(|(_, q)| q).sum();

    if bid > ask * 12 / 10 {
        Side::Buy
    } else if ask > bid * 12 / 10 {
        Side::Sell
    } else {
        Side::Hold
    }
}