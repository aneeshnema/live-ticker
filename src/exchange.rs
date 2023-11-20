pub mod binance;
pub mod okx;

use anyhow::{Context, Error, Result};
use tokio::sync::mpsc::Sender;

#[derive(Debug, PartialEq)]
pub struct Level {
    pub price: f32,
    pub quantity: f32,
}

impl Level {
    pub fn new(vec: &[String]) -> Result<Level> {
        return Ok(Level {
            price: vec[0].parse::<f32>().context("failed to parse price")?,
            quantity: vec[1].parse::<f32>().context("failed to parse quantity")?,
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct Orderbook {
    pub exchange: String,
    pub bids: Option<Vec<Level>>,
    pub asks: Option<Vec<Level>>,
}

pub trait Exchange<T> {
    async fn start(&mut self, token_pair: &str, sender: Sender<Orderbook>);
    fn get_orderbook(exch: &T) -> Result<Option<Orderbook>, Error>;
}
