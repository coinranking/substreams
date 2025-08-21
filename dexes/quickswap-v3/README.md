# QuickSwap V3 (Algebra) Substreams

Real-time block-aggregated swap data for QuickSwap V3 pools on Polygon mainnet.

## Overview

This substreams package tracks QuickSwap V3 activity on Polygon. QuickSwap V3 uses the Algebra protocol, which is a fork of Uniswap V3 with additional features like dynamic fees.

## Key Addresses

- **Algebra Factory**: `0x411b0fAcC3489691f28ad58c47006AF5E3Ab3A28`
- **Network**: Polygon (Matic) Mainnet

## Modules

### `map_quickswap_pools_created`
Tracks pool creation events from the Algebra Factory contract.

### `map_quickswap_ticker_output`
Aggregates swap data per pool per block, providing:
- Block volume for token0 and token1
- Number of swaps
- Closing price
- Timestamp

## Differences from Uniswap V3

QuickSwap V3 uses the Algebra protocol which has several key differences:

1. **Dynamic Fees**: Unlike Uniswap V3's fixed fee tiers (0.05%, 0.3%, 1%), Algebra uses dynamic fees that adjust based on market conditions
2. **Event Signatures**: Different event signatures with indexed parameters
3. **Price Format**: Uses Q64.96 format for prices

## Building

```bash
cargo build --release --target wasm32-unknown-unknown
substreams pack
```

## Usage

```bash
substreams run coinranking-quickswap-v3-v0.1.0.spkg map_quickswap_ticker_output \
  --substreams-api-token="YOUR_TOKEN" \
  --substreams-endpoint="polygon.streamingfast.io:443" \
  -s 65000000  # Start from any block
```

Note: Replace `YOUR_TOKEN` with your actual StreamingFast API token.
Get your token at: https://app.streamingfast.io/