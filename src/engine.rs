use crossbeam_channel::bounded;
use crate::{market, strategy, trader, risk};

pub async fn run() {
    let (tx, rx) = bounded(256);
    tokio::spawn(market::start(tx));

    let mut position = 0i64;

    while let Ok(book) = rx.recv(){
        let side = strategy::decide(&book);

        if risk::allow(&side, position) {
            trader::execute(side).await;
        }
    }
}