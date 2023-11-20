use anyhow::{Error, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::exchange::{Exchange, Level, Orderbook};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Binance {
    bids: Option<Vec<Vec<String>>>,
    asks: Option<Vec<Vec<String>>>,
}

impl Binance {
    pub fn new() -> Self {
        Binance {
            bids: None,
            asks: None,
        }
    }
}

impl Exchange<Binance> for Binance {
    async fn start(&mut self, _token_pair: &str, sender: Sender<Orderbook>) {
        let token_pair = "ethbtc";
        let url = format!("wss://stream.binance.com:9443/ws/{token_pair}@depth10@100ms");

        tokio::spawn(async move {
            loop {
                let (_, mut ws_read) = match connect_async(&url).await {
                    Ok((stream, _)) => stream.split(),
                    Err(err) => {
                        eprintln!("Connection failure: {err}");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                while let Some(msg) = ws_read.next().await {
                    let Ok(Message::Text(text)) = msg else {
                        continue;
                    };
                    match serde_json::from_str::<Binance>(&text) {
                        Ok(value) => match Binance::get_orderbook(&value) {
                            Ok(Some(book)) => {
                                let _ = sender.send(book).await;
                            }
                            Ok(_) => (),
                            Err(err) => eprintln!("{err}"),
                        },
                        Err(err) => eprintln!("Parsing failure: {err}"),
                    }
                }
            }
        });
    }

    fn get_orderbook(exch: &Self) -> Result<Option<Orderbook>, Error> {
        let bids = match &exch.bids {
            Some(levels) => Some(
                levels
                    .iter()
                    .map(|level| Level::new(level))
                    .collect::<Result<Vec<Level>, Error>>()?,
            ),
            None => None,
        };
        let asks = match &exch.asks {
            Some(levels) => Some(
                levels
                    .iter()
                    .map(|level| Level::new(level))
                    .collect::<Result<Vec<Level>, Error>>()?,
            ),
            None => None,
        };
        return Ok(Some(Orderbook {
            exchange: "binance".into(),
            bids,
            asks,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json() {
        let json = r#"{
            "lastUpdateId":123456789,
            "bids":[
                ["100.0","10.0"],
                ["90.0","22.0"]
            ],
            "asks":[
                ["100.5","5.1"],
                ["101.23","13.33"]
            ]
        }"#;

        match serde_json::from_str::<Binance>(json) {
            Ok(binance) => {
                let Ok(Some(orderbook)) = Binance::get_orderbook(&binance) else {
                    panic!();
                };
                assert_eq!(
                    orderbook,
                    Orderbook {
                        exchange: "binance".into(),
                        bids: Some(vec![
                            Level {
                                price: 100.0,
                                quantity: 10.0
                            },
                            Level {
                                price: 90.0,
                                quantity: 22.0
                            }
                        ]),
                        asks: Some(vec![
                            Level {
                                price: 100.5,
                                quantity: 5.1
                            },
                            Level {
                                price: 101.23,
                                quantity: 13.33
                            }
                        ]),
                    }
                );
            }
            Err(err) => panic!("{err}"),
        }
    }
}
