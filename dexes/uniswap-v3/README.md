# Uniswap V3 24-Hour Rolling Volume Substream

Real-time 24-hour rolling volume tracking for Uniswap V3 pools on Ethereum mainnet.

## Features

- Tracks pool creation events with token decimals
- Captures all swap transactions with closing prices
- Calculates 24-hour rolling volumes using efficient 5-minute aggregation
- Provides both per-block and 24-hour volume metrics

## Output Format

The substream produces `DexOutput` messages containing:

- **pools_created**: New pool deployments with token addresses and decimals
- **tickers**: Per-pool metrics including:
  - Block-level volumes for both tokens
  - Swap count
  - Closing price (raw sqrtPriceX96 format)
  - 24-hour rolling volumes

## Usage

```bash
substreams run coinranking-uniswap-v3-v0.1.0.spkg map_uniswap_ticker_output \
  --substreams-api-token="YOUR_TOKEN" \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  -s -7500  # Start ~25 hours ago for complete 24h data
```

## Notes

- Prices are in raw format - apply decimal adjustment using token decimals from pool creation events
- All Ethereum addresses include the `0x` prefix
- Volume tracking uses a circular buffer for efficient memory usage