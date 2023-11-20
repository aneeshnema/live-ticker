mod exchange;

use exchange::binance::Binance;
use exchange::okx::Okx;
use exchange::{Exchange, Orderbook};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel::<Orderbook>(32);

    let mut binance = Binance::new();
    binance.start("ethbtc", sender.clone()).await;

    let mut okx = Okx {};
    okx.start("ETH-BTC", sender.clone()).await;

    loop {
        match receiver.recv().await {
            Some(orderbook) => println!("{:?}", orderbook),
            None => (),
        }
    }
}
