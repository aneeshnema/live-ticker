pub mod binance;
pub mod okx;

use crate::live_ticker::Venue;
use anyhow::{Error, Result};
use tokio::sync::mpsc::Sender;

pub trait Exchange<T> {
    async fn start(&mut self, token_pair: &str, sender: Sender<Venue>);
    fn get_l1_data(exch: &T) -> Result<Option<Venue>, Error>;
}
