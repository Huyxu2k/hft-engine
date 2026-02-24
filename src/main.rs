mod core;
mod engine;
mod execute;
mod risk;
mod trader;
mod types;

use crate::engine::start_core_engine;

fn main() {
    let _cmd_tx = start_core_engine();
    println!("Core engine started");
}
