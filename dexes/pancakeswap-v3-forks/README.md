# PancakeSwap V3 Substreams

Real-time block-aggregated swap data for PancakeSwap V3 across multiple blockchains.

## Overview

This substreams package is specifically designed for PancakeSwap V3, which uses a **different Swap event signature** than Uniswap V3.

Supported networks:
- **BNB Chain** (BSC) - Primary PancakeSwap V3 deployment
- **Ethereum** - Cross-chain deployment
- **Arbitrum** - Layer 2 deployment
- **zkSync Era** - Layer 2 deployment
- **Polygon zkEVM** - Layer 2 deployment

## Key Difference from Uniswap V3

PancakeSwap V3 uses an extended Swap event that includes protocol fee tracking:
```
Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick, uint128 protocolFeesToken0, uint128 protocolFeesToken1)
```

**Event Signature Hash:** `0x19b47279256b2a23a1665c810c8d55a1758940ee09377d4f8d26497a3577dc83`

This is different from Uniswap V3's signature (`0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67`), which means a separate substreams package is required.

## Network Configuration

The target network is specified at runtime via CLI parameters:

### BNB Chain (BSC)
```bash
substreams run coinranking-pancakeswap-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="bsc.streamingfast.io:443" \
  --start-block=26324014
```

### Ethereum Mainnet
```bash
substreams run coinranking-pancakeswap-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  --start-block=16950686
```

### Arbitrum One
```bash
substreams run coinranking-pancakeswap-v3-v0.1.0.spkg map_v3_ticker_output \
  --substreams-endpoint="arb-one.streamingfast.io:443" \
  --start-block=<TBD>
```

## PancakeSwap V3 Deployment Blocks

| Network | Deployment Block | Factory Address |
|---------|-----------------|-----------------|
| BSC | 26324014 | 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865 |
| Ethereum | 16950686 | 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865 |
| Arbitrum | TBD | 0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865 |
| zkSync Era | TBD | TBD |
| Polygon zkEVM | TBD | TBD |

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