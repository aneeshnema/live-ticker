use anyhow::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::exchange::{Exchange, Level, Orderbook};

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
    async fn start(&mut self, _token_pair: &str, sender: Sender<Orderbook>) {
        let url = "wss://ws.okx.com:8443/ws/v5/public";
        // let url = "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999";
        let token_pair = "ETH-BTC";

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
                        // println!("Received: {}", text);
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
                    // println!("Received: {}", text);
                    match serde_json::from_str::<OkxData>(&text) {
                        Ok(value) => match Okx::get_orderbook(&value) {
                            Ok(Some(book)) => {
                                let _ = sender.send(book).await;
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

    fn get_orderbook(data: &OkxData) -> Result<Option<Orderbook>, Error> {
        match data {
            OkxData::Push { data } => {
                let Some(ask_bid) = data.first() else {
                    return Ok(None);
                };
                let asks = ask_bid
                    .asks
                    .iter()
                    .map(|level| Level::new(level))
                    .collect::<Result<Vec<Level>, Error>>()?;
                let bids = ask_bid
                    .bids
                    .iter()
                    .map(|level| Level::new(level))
                    .collect::<Result<Vec<Level>, Error>>()?;
                return Ok(Some(Orderbook {
                    exchange: "okx".into(),
                    bids: Some(bids),
                    asks: Some(asks),
                }));
            }
            _ => return Ok(None),
        }
    }
}
