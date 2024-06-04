use ac_primitives::DefaultRuntimeConfig;
use clap::Parser;
use sp_core::H256;
use substrate_api_client::{rpc::WsRpcClient, Api, GetChainInfo, GetStorage};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The network to connect to ( e.g 'harmonie' for the testnet )
    /// You can also provide the full URL of the node.
    #[arg(short, long, default_value = "harmonie")]
    network: String,

    // Define the number of blocks to analyze
    #[arg(long, default_value_t = 1000)]
    num_blocks: u32,
}

fn main() {
    let args = Args::parse();
    let url = match args.network.as_str() {
        "harmonie" => "wss://harmonie-endpoint-02.allfeat.io",
        _ => &args.network,
    };

    let client = WsRpcClient::new(url).unwrap();
    let api = Api::<DefaultRuntimeConfig, _>::new(client).unwrap();

    // Define the number of blocks to analyze
    let num_blocks: u32 = args.num_blocks;

    // Fetch the latest finalized block number
    let latest_block_hash = api.get_finalized_head().unwrap().unwrap();
    let latest_block = api.get_header(Some(latest_block_hash)).unwrap().unwrap();
    let latest_block_number = latest_block.number;

    let mut timestamps = vec![];
    let mut tx_counts = vec![];

    // Loop through the blocks to collect timestamps and transaction counts
    for i in 0..num_blocks {
        let block_number = latest_block_number - i;
        let block_hash = api.get_block_hash(Some(block_number)).unwrap().unwrap();

        println!(
            "Fetching block: {} with hash {:?} as block number {}",
            block_number, block_hash, i
        );
        let block = api.get_signed_block(Some(block_hash)).unwrap().unwrap();

        // Extract timestamp and transaction count
        let timestamp = get_block_timestamp(&api, block_hash);
        let tx_count = block.block.extrinsics.len() as u32;
        println!("Block timestamp: {:?}, tx count: {}", timestamp, tx_count);

        if let Some(ts) = timestamp {
            timestamps.push(ts);
            tx_counts.push(tx_count);
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
}

fn get_block_timestamp(
    api: &Api<DefaultRuntimeConfig, WsRpcClient>,
    block_hash: H256,
) -> Option<u64> {
    let storage_data: Option<Vec<u8>> = api
        .get_storage_by_key(
            ac_primitives::StorageKey(
                "0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb".into(),
            ),
            Some(block_hash),
        )
        .unwrap();

    println!("Storage data: {:?}", storage_data);
    if let Some(data) = storage_data {
        if data.len() >= 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&data[..8]);
            let timestamp = u64::from_le_bytes(arr);
            Some(timestamp)
        } else {
            None
        }
    } else {
        None
    }
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
