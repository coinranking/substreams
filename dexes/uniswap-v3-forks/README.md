# Uniswap V3 and Compatible Forks Substreams

Real-time block-aggregated swap data for Uniswap V3 and all compatible forks across multiple blockchains.

## Overview

This substreams package provides a universal implementation that works with all Uniswap V3 forks, including:

- **Uniswap V3** - Ethereum, Polygon, Arbitrum, Optimism, Base, Celo, BNB Chain
- **QuickSwap V3 (Algebra Protocol)** - Polygon  
- **PancakeSwap V3** - BNB Chain, Ethereum, Arbitrum, zkSync Era, Polygon zkEVM
- **SushiSwap V3** - Multiple chains
- **Any other V3 fork** that maintains compatible event signatures

## How It Works

This implementation tracks the standard Swap event emitted by all V3 pools:
```
Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)
```

Since all V3 forks use the same event structure, this single implementation can process swaps from any V3 deployment on any EVM-compatible blockchain.

## Network Configuration

The target network is specified at runtime via CLI parameters:

### Ethereum Mainnet
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  --start-block=12369621  # Uniswap V3 deployment
```

### Polygon
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="polygon.streamingfast.io:443" \
  --start-block=22757547  # Uniswap V3 on Polygon
```

### Arbitrum One
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="arb-one.streamingfast.io:443" \
  --start-block=1107  # Uniswap V3 on Arbitrum
```

### Optimism
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="opt-mainnet.streamingfast.io:443" \
  --start-block=10028767  # Uniswap V3 on Optimism
```

### Base
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="base-mainnet.streamingfast.io:443" \
  --start-block=1371680  # Uniswap V3 on Base
```

### BNB Chain
```bash
substreams run coinranking-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="bnb.streamingfast.io:443" \
  --start-block=26324014  # PancakeSwap V3 on BSC
```

## Key Deployment Blocks

| Protocol | Network | Deployment Block | Factory Address |
|----------|---------|-----------------|-----------------|
| Uniswap V3 | Ethereum | 12369621 | 0x1F98431c8aD98523631AE4a59f267346ea31F984 |
| Uniswap V3 | Polygon | 22757547 | 0x1F98431c8aD98523631AE4a59f267346ea31F984 |
| Uniswap V3 | Arbitrum | 1107 | 0x1F98431c8aD98523631AE4a59f267346ea31F984 |
| Uniswap V3 | Optimism | 10028767 | 0x1F98431c8aD98523631AE4a59f267346ea31F984 |
| Uniswap V3 | Base | 1371680 | 0x33128a8fC17869897dcE68Ed026d694621f6FDfD |
| QuickSwap V3 | Polygon | 29000000 | 0x411b0fAcC3489691f28ad58c47006AF5E3Ab3A28 |
| PancakeSwap V3 | BSC | 26324014 | 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865 |
| PancakeSwap V3 | Ethereum | 16950686 | 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865 |

## Module

### `map_v3_ticker_output`
Aggregates swap data per pool per block, providing:
- Block volume for token0 and token1 (in raw token units)
- Number of swaps
- Closing price (calculated from sqrtPriceX96)
- Block number and timestamp

## Output Format

This implementation outputs a `TickerOutput` message containing aggregated swap data for each pool that had activity in the block:

### Volume Data
- **Volumes are raw token units** (not decimal-adjusted)
- Example: 500 USDC (6 decimals) is reported as "500000000"
- Consumers must divide by 10^decimals to get actual token amounts

### Price Data
- **Price is provided as sqrtPriceX96** - the raw value from swap events
- This is NOT a human-readable price
- Clients must calculate the actual price using:
  ```
  price = (sqrtPriceX96 / 2^96)^2 * 10^(token0_decimals - token1_decimals)
  ```
- The sqrtPriceX96 format is standard across all Uniswap V3 forks

## Building

```bash
cargo build --release --target wasm32-unknown-unknown
substreams pack
```

## Testing

Run the test script with any supported network:
```bash
./test.sh -s <start_block> -e <stop_block>
```

## Requirements

- Substreams CLI
- Rust toolchain with wasm32-unknown-unknown target
- StreamingFast API token (get yours at https://app.streamingfast.io/)