mod exchange;
mod venue_aggregator;

use clap::Parser;
use futures_util::StreamExt;
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tonic::{Request, Response, Status};

pub mod live_ticker {
    tonic::include_proto!("live_ticker");
}

use exchange::binance::Binance;
use exchange::okx::Okx;
use exchange::Exchange;
use live_ticker::l1_ticker_server::{L1Ticker, L1TickerServer};
use live_ticker::{Empty, L1Summary, Venue};
use venue_aggregator::VenueAggregator;

/// Simple program to fetch Order book of crypto token from exchanges and display them on CLI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Token pair to fetch MD of, eg, ETH-USD
    #[arg(short, long, default_value = "ETH-USDT")]
    token_pair: String,

    /// Port to start the server on
    #[arg(short, long, default_value_t = 7777)]
    port: u16,
}

#[derive(Debug)]
struct L1TickerService {
    receiver_summary: broadcast::Receiver<L1Summary>,
}

#[tonic::async_trait]
impl L1Ticker for L1TickerService {
    type GetL1SummaryStream =
        Pin<Box<dyn Stream<Item = Result<L1Summary, Status>> + Send + 'static>>;

    async fn get_l1_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetL1SummaryStream>, Status> {
        let receiver = self.receiver_summary.resubscribe();

        let result = BroadcastStream::new(receiver)
            .filter_map(|r| {
                // the stream is of type Result<L1Summary, BroadcastStreamRecvError>
                // we need to convert it into Result<L1Summary, Status>
                std::future::ready(match r {
                    Ok(r) => Some(Ok(r)),
                    _ => None, // errors ignored
                })
            })
            .boxed();
        return Ok(Response::new(result as Self::GetL1SummaryStream));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let binance_token = args
        .token_pair
        .split('-')
        .map(|x| x.to_lowercase())
        .collect::<Vec<String>>()
        .join("");
    let okx_token = args.token_pair.clone();

    let (sender_venue, receiver_venue) = mpsc::channel::<Venue>(32);
    let (sender_summary, receiver_summary) = broadcast::channel::<L1Summary>(32);

    let mut binance = Binance::new();
    binance
        .start(&binance_token[..], sender_venue.clone())
        .await;

    let mut okx = Okx {};
    okx.start(&okx_token[..], sender_venue.clone()).await;

    VenueAggregator::start(receiver_venue, sender_summary).await;

    let service = L1TickerService { receiver_summary };

    let server = L1TickerServer::new(service);

    // Start GRPC server
    tonic::transport::Server::builder()
        .add_service(server)
        .serve(std::net::SocketAddr::from(([127, 0, 0, 1], args.port)))
        .await?;

    Ok(())
}
