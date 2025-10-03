# Uniswap V2 and Compatible Forks Substreams

Real-time block-aggregated swap data for Uniswap V2 and all compatible forks across multiple blockchains.

## Overview

This substreams package provides a universal implementation that works with all Uniswap V2 forks, including:

- **Uniswap V2** - Ethereum
- **SushiSwap** - Ethereum, Polygon, Arbitrum, Avalanche, BSC, and more
- **PancakeSwap V2** - BNB Chain, Ethereum
- **QuickSwap V2** - Polygon
- **Any other V2 fork** that maintains compatible event signatures

## How It Works

This implementation tracks the standard Swap and Sync events emitted by all V2 pools:

```
Swap(address indexed sender, uint amount0In, uint amount1In, uint amount0Out, uint amount1Out, address indexed to)
Sync(uint112 reserve0, uint112 reserve1)
```

Since all V2 forks use the same event structure, this single implementation can process swaps from any V2 deployment on any EVM-compatible blockchain.

## Network Configuration

The target network is specified at runtime via CLI parameters. You must provide a start block.

### Ethereum Mainnet
```bash
substreams run coinranking-v2-v0.1.0.spkg map_v2_ticker_output \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  --start-block=10000835  # Uniswap V2 deployment
```

### Polygon
```bash
substreams run coinranking-v2-v0.1.0.spkg map_v2_ticker_output \
  --substreams-endpoint="polygon.streamingfast.io:443" \
  --start-block=4931780  # SushiSwap on Polygon
```

### BSC
```bash
substreams run coinranking-v2-v0.1.0.spkg map_v2_ticker_output \
  --substreams-endpoint="bnb.streamingfast.io:443" \
  --start-block=6809737  # PancakeSwap V2 on BSC
```

## Key Deployment Blocks

| Protocol | Network | Deployment Block | Factory Address |
|----------|---------|-----------------|-----------------|
| Uniswap V2 | Ethereum | 10000835 | 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f |
| SushiSwap | Ethereum | 10794229 | 0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac |
| SushiSwap | Polygon | 4931780 | 0xc35DADB65012eC5796536bD9864eD8773aBc74C4 |
| SushiSwap | Arbitrum | 70 | 0xc35DADB65012eC5796536bD9864eD8773aBc74C4 |
| PancakeSwap V2 | BSC | 6809737 | 0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73 |
| QuickSwap V2 | Polygon | 4931900 | 0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32 |

## Module

### `map_v2_ticker_output`
Aggregates swap data per pool per block, providing:
- Block volume for token0 and token1 (in raw token units)
- Number of swaps
- Current reserves (reserve0 and reserve1 from Sync events)
- Block number and timestamp

## Output Format

This implementation outputs a `TickerOutput` message containing aggregated swap data for each pool that had activity in the block:

### Volume Data
- **Volumes are raw token units** (not decimal-adjusted)
- Example: 500 USDC (6 decimals) is reported as "500000000"
- Consumers must divide by 10^decimals to get actual token amounts

### Reserve Data
- **Reserves are provided as uint112 values** from Sync events
- These represent the current pool reserves at the end of the block
- Price can be calculated as: `reserve1 / reserve0` (after decimal adjustment)
- Example calculation:
  ```
  # For a WETH/USDC pool with:
  # reserve0 = 100000000000000000000 (100 WETH, 18 decimals)
  # reserve1 = 150000000000 (150000 USDC, 6 decimals)

  price_usdc_per_weth = (reserve1 / 10^6) / (reserve0 / 10^18)
                      = 150000 / 100
                      = 1500 USDC per WETH
  ```

## Building

```bash
cargo build --release --target wasm32-unknown-unknown
substreams pack
```

## Testing

Run the test script with a specific start block:
```bash
./test.sh -s <start_block> -e <stop_block>
```

Example:
```bash
./test.sh -s 10000835 -e 10000850  # Test first 15 blocks after Uniswap V2 deployment
```

## Requirements

- Substreams CLI
- Rust toolchain with wasm32-unknown-unknown target
- StreamingFast API token (get yours at https://app.streamingfast.io/)
