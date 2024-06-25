use clap::Parser;
use std::collections::VecDeque;
use subxt::{PolkadotConfig};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The network to connect to ( e.g 'harmonie' for the testnet )
    /// You can also provide the full URL of the node.
    #[arg(short, long, default_value = "harmonie")]
    network: String,

    // Define the number of blocks to analyze
    #[arg(long, default_value_t = 10)]
    num_blocks: usize,
}

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod harmonie {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the endpoint URL
    let args = Args::parse();
    let url = match args.network.as_str() {
        "harmonie-testnet" => "wss://harmonie-endpoint-02.allfeat.io",
        _ => &args.network,
    };

    // Build the client
    let client = subxt::client::OnlineClient::<PolkadotConfig>::from_url(url).await?;
    println!("Connection with {url} established.");

    let block_client = client.blocks();
    let mut block_stream = block_client.subscribe_finalized().await?;

    // Define the number of blocks to analyze
    let num_blocks = args.num_blocks;
    let mut timestamps = VecDeque::with_capacity(num_blocks);
    let mut tx_counts = VecDeque::with_capacity(num_blocks);
    let storage_key = "0x26aa394eea5630e07c48ae0c9558cef7".as_bytes();

    while let Some(block) = block_stream.next().await {
        let block = block?;
        println!("New block: {}", block.hash());

        // Fetch the timestamp from the block
        if let Some(block_detail) = block.extrinsics().await?.iter().nth(0) {
            let block_detail = block_detail?;
            let timestamp = block_detail.;
            let tx_count = block.extrinsics().await?.len() as u64;
            timestamps.push_back(timestamp);
            tx_counts.push_back(tx_count);

            println!(
                "Block #{}: Timestamp: {}, Extrinsics: {}",
                block.hash(),
                timestamp,
                tx_count
            );
        }

        if timestamps.len() >= num_blocks {
            break;
        }
    }

    // Calculate TPS
    if timestamps.len() > 1 {
        let mut tps_values = vec![];
        for i in 1..timestamps.len() {
            let time_diff = (timestamps[i] - timestamps[i - 1]) as f64 / 1000.0;
            let tx_diff = tx_counts[i] as f64;
            let tps = tx_diff / time_diff;
            tps_values.push(tps);
        }
        let median_tps = median(&mut tps_values);
        println!("Median TPS: {:.2}", median_tps);
    } else {
        println!("Not enough data to calculate TPS");
    }

    Ok(())
}

// Function to calculate the median of a vector
fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}
