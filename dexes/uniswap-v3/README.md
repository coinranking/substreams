# Uniswap V3 Block-Aggregated Swap Data Substream

Real-time block-aggregated swap data for Uniswap V3 pools on Ethereum mainnet.

## Features

- Tracks pool creation events with token addresses and decimals
- Aggregates all swaps per pool per block
- Provides closing prices and swap counts
- Stateless design with no persistent stores

## Output Format

The substream produces `DexOutput` messages containing:

- **pools_created**: New pool deployments with token addresses and decimals
- **tickers**: Per-pool metrics including:
  - Block-level volumes for both tokens
  - Swap count
  - Closing price (token1/token0 ratio)
  - Block number and timestamp

## Usage

```bash
substreams run coinranking-uniswap-v3-v0.2.0.spkg map_uniswap_ticker_output \
  --substreams-api-token="YOUR_TOKEN" \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  -s 12345678  # Start from any block
```

## Notes

- Prices are calculated from sqrtPriceX96 format to standard ratio
- All Ethereum addresses include the `0x` prefix
- No stores or state management - purely block-level aggregation
- Downstream systems handle rolling window calculations

## Changelog

### v0.2.0
- **BREAKING**: Removed 24-hour rolling volume calculations and all stores
- Simplified to stateless block-aggregated output only
- Removed `volume_24h_token0` and `volume_24h_token1` fields from output
- Eliminated store size limitations and potential isolation issues
- Simplified codebase structure - all logic now in single `lib.rs` file

### v0.1.2
- Fixed critical bug where negative volumes could occur during the first 24 hours of processing
- Added proper guard to prevent volume eviction before 24 hours of data has accumulated
- This ensures each period's volume is added exactly once and subtracted exactly once