use anyhow::{Context, Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{exchange::Exchange, live_ticker::Venue};

#[derive(Deserialize, Debug)]
pub struct AskBid {
    asks: Vec<Vec<String>>,
    bids: Vec<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OkxData {
    Push { data: Vec<AskBid> },
    Response { event: String },
}

pub struct Okx {}

impl Exchange<OkxData> for Okx {
    async fn start(&mut self, token_pair: &str, sender: Sender<Venue>) {
        let url = "wss://ws.okx.com:8443/ws/v5/public";
        // let url = "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999";
        let token_pair = token_pair.to_owned();

        tokio::spawn(async move {
            loop {
                let mut stream = match connect_async(url).await {
                    Ok((stream, _)) => stream,
                    Err(err) => {
                        eprintln!("Connection failure: {err}");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                match stream
                    .send(Message::Text(
                        format!("{{\"op\": \"subscribe\",\"args\": [{{\"channel\": \"books5\",\"instId\": \"{token_pair}\"}}]}}"),
                    ))
                    .await
                {
                    Err(err) => {eprintln!("{err}"); continue;},
                    Ok(_) => {
                        let Some(Ok(Message::Text(text))) = stream.next().await else {
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            continue;
                        };
                        match serde_json::from_str::<OkxData>(&text) {
                            Err(err) => eprintln!("Subscription failure: {err}"),
                            Ok(OkxData::Response { event }) => {
                                if event != "subscribe" {
                                    eprintln!("Something wrong with subscription response: {text}");
                                }
                            }
                            Ok(_) => eprintln!("Something wrong with subscription response: {text}")
                        }
                    },
                };

                while let Some(msg) = stream.next().await {
                    let Ok(Message::Text(text)) = msg else {
                        continue;
                    };
                    match serde_json::from_str::<OkxData>(&text) {
                        Ok(value) => match Okx::get_l1_data(&value) {
                            Ok(Some(venue)) => {
                                let _ = sender.send(venue).await;
                            }
                            Ok(_) => (),
                            Err(err) => eprintln!("{err}"),
                        },
                        Err(err) => {
                            eprintln!("Parsing failure: {err} | Received text: {text}");
                        }
                    }
                }
            }
        });
    }

    fn get_l1_data(data: &OkxData) -> Result<Option<Venue>, Error> {
        match data {
            OkxData::Push { data } => {
                let Some(ask_bid) = data.first() else {
                    return Ok(None);
                };
                let ask_data = ask_bid.asks[0]
                    .iter()
                    .map(|value| {
                        value
                            .parse::<f64>()
                            .context("failed to parse string into float")
                    })
                    .collect::<Result<Vec<f64>, Error>>()?;
                // .map(|level| Level::new(level))
                // .collect::<Result<Vec<Level>, Error>>()?;
                let bid_data = ask_bid.bids[0]
                    .iter()
                    .map(|value| {
                        value
                            .parse::<f64>()
                            .context("failed to parse string into float")
                    })
                    .collect::<Result<Vec<f64>, Error>>()?;
                // .map(|level| Level::new(level))
                // .collect::<Result<Vec<Level>, Error>>()?;
                return Ok(Some(Venue {
                    name: String::from("okx"),
                    bid_price: bid_data[0],
                    bid_size: bid_data[1],
                    ask_price: ask_data[0],
                    ask_size: ask_data[1],
                }));
            }
            _ => return Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_response_json() {
        let json = r#"{"event":"subscribe","arg":{"channel":"books5","instId":"ETH-BTC"},"connId":"b5ceefd8"}"#;

        match serde_json::from_str::<OkxData>(json) {
            Ok(OkxData::Response { event }) => assert_eq!(event, "subscribe"),
            Ok(_) => panic!("data not parsed into Response"),
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn parse_push_json() {
        let json = r#"{
            "arg":{"channel":"books5","instId":"ETH-BTC"},
            "data":[
                {
                    "asks":[
                        ["100.5","5.1","0","1"],
                        ["101.23","13.33","0","2"]
                    ],
                    "bids":[
                        ["100.0","10.0","0","9"],
                        ["90.0","22.0","0","8"]
                    ],
                    "instId":"ETH-BTC",
                    "ts":"1700582054108",
                    "seqId":1062841525
                }
            ]
        }"#;

        match serde_json::from_str::<OkxData>(json) {
            Ok(okx) => {
                let Ok(Some(venue)) = Okx::get_l1_data(&okx) else {
                    panic!();
                };
                assert_eq!(
                    venue,
                    Venue {
                        name: String::from("okx"),
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
