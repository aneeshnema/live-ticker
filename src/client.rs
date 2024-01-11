use std::str::FromStr;

use clap::Parser;
use tonic::Request;

pub mod live_ticker {
    tonic::include_proto!("live_ticker");
}

use live_ticker::l1_ticker_client::L1TickerClient;
use live_ticker::Empty;
use tonic::transport::Uri;

/// Simple GRPC client to L1 summary from server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// port the server is running on
    #[arg(short, long, default_value_t = 7777)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create a gRPC client
    let channel = tonic::transport::Channel::builder(
        Uri::from_str(format!("http://127.0.0.1:{}", args.port).as_str()).unwrap(),
    )
    .connect()
    .await?;
    let mut client = L1TickerClient::new(channel);

    // Make a request to start the streaming
    let request = Request::new(Empty {});
    let mut stream = client.get_l1_summary(request).await?.into_inner();

    // Process incoming stream items
    while let Some(response) = stream.message().await? {
        println!("Received: {:?}", response);
    }

    Ok(())
}
