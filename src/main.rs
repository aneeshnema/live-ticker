mod exchange;

use clap::Parser;
use exchange::binance::Binance;
use exchange::okx::Okx;
use exchange::{Exchange, Orderbook};
use tokio::sync::mpsc;

/// Simple program to fetch Order book of crypto token from exchanges and display them on CLI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Token pair to fetch MD of, eg, ETH-USD
    token_pair: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let binance_token = args
        .token_pair
        .split('-')
        .map(|x| x.to_lowercase())
        .collect::<Vec<String>>()
        .join("");
    let okx_token = args.token_pair.clone();

    let (sender, mut receiver) = mpsc::channel::<Orderbook>(32);

    let mut binance = Binance::new();
    binance.start(&binance_token[..], sender.clone()).await;

    let mut okx = Okx {};
    okx.start(&okx_token[..], sender.clone()).await;

    loop {
        match receiver.recv().await {
            Some(orderbook) => println!("{:?}", orderbook),
            None => (),
        }
    }
}
