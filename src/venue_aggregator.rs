use std::collections::HashMap;

use crate::live_ticker::{L1Summary, Venue};
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub struct VenueAggregator {}

impl VenueAggregator {
    pub async fn start(mut receiver: mpsc::Receiver<Venue>, sender: broadcast::Sender<L1Summary>) {
        tokio::spawn(async move {
            let mut venue_map: HashMap<String, Venue> = HashMap::new();

            loop {
                match receiver.recv().await {
                    Some(venue) => {
                        // println!("{:?}", venue);
                        venue_map.insert(venue.name.clone(), venue);
                        let venues = venue_map.values().cloned().collect::<Vec<Venue>>();
                        _ = sender.send(L1Summary { venues });
                    }
                    None => (),
                }
            }
        });
    }
}
