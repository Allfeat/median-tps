# Median tps script

This script is made for calculating the median number of transactions per second (tps) on a subsrate based blockchain (allfeat testnet by default).

## Usage

To run the script on allfeat testnet, simply run the following command:

```bash
cargo run
```

To run the script on a custom chain, run the following command:

```bash
cargo run -- -n <chain-wss-url>
```

If you want to run the script with a custom number of blocks to analyze, run the following command:

```bash
cargo run -- --num-blocks <number-of-blocks>
```
