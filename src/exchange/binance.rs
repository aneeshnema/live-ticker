use anyhow::{Context, Error, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{exchange::Exchange, live_ticker::Venue};

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
    async fn start(&mut self, token_pair: &str, sender: Sender<Venue>) {
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
                        Ok(value) => match Binance::get_l1_data(&value) {
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

    fn get_l1_data(exch: &Self) -> Result<Option<Venue>, Error> {
        let bid_data = match &exch.bids {
            Some(levels) => Some(
                levels[0]
                    .iter()
                    .map(|value| {
                        value
                            .parse::<f64>()
                            .context("failed to parse string into float")
                    })
                    .collect::<Result<Vec<f64>, Error>>()?,
                // levels
                //     .iter()
                //     .map(|level| Level::new(level))
                //     .collect::<Result<Vec<Level>, Error>>()?,
            ),
            None => None,
        };
        let ask_data = match &exch.asks {
            Some(levels) => Some(
                levels[0]
                    .iter()
                    .map(|value| {
                        value
                            .parse::<f64>()
                            .context("failed to parse string into float")
                    })
                    .collect::<Result<Vec<f64>, Error>>()?,
                // levels
                //     .iter()
                //     .map(|level| Level::new(level))
                //     .collect::<Result<Vec<Level>, Error>>()?,
            ),
            None => None,
        };

        let mut venue = Venue {
            name: String::from("binance"),
            bid_price: 0.0,
            bid_size: 0.0,
            ask_price: 0.0,
            ask_size: 0.0,
        };

        match bid_data {
            Some(arr) => {
                venue.bid_price = arr[0];
                venue.bid_size = arr[1];
            }
            _ => (),
        }

        match ask_data {
            Some(arr) => {
                venue.ask_price = arr[0];
                venue.ask_size = arr[1];
            }
            _ => (),
        }
        return Ok(Some(venue));
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
                let Ok(Some(venue)) = Binance::get_l1_data(&binance) else {
                    panic!();
                };
                assert_eq!(
                    venue,
                    Venue {
                        name: String::from("binance"),
                        bid_price: 100.0,
                        bid_size: 10.0,
                        ask_price: 100.5,
                        ask_size: 5.1
                    }
                );
            }
            Err(err) => panic!("{err}"),
        }
    }
}
