# DEX Ticker Data Substreams

A collection of Substreams packages for tracking real-time, block-aggregated trading data across decentralized exchanges (DEXes).

## Project Structure

```
substreams/
├── proto/                      # Common protocol buffer definitions
│   └── dex_common.proto       # Universal ticker format for all DEXes
├── common/                     # Shared utilities and functions
└── dexes/                      # Individual DEX implementations
    └── uniswap-v3-forks/      # Universal Uniswap V3 and forks implementation
```

## Universal Output Format

All DEX packages output the same `TickerOutput` format defined in `proto/dex_common.proto`. This universal format ensures:
- Consistent data structure across ALL DEX types (V2, V3, Curve, etc.)
- Easy aggregation of volumes across multiple DEXes
- Unified downstream data pipelines
- Simple integration of new DEXes

The ticker format provides block-aggregated data including:
- Token volumes (raw units)
- Swap counts
- Closing prices
- Block number and timestamp

## Supported DEXes

### Current
- ✅ **Uniswap V3 and all forks** - Universal implementation working on:
  - Uniswap V3 (Ethereum, Polygon, Arbitrum, Optimism, Base, etc.)
  - QuickSwap V3 (Polygon)
  - PancakeSwap V3 (BSC, Ethereum, Arbitrum)
  - SushiSwap V3
  - Any other V3 fork

### Planned
- 🔜 Uniswap V2 and forks (PancakeSwap V2, SushiSwap V2, etc.)
- 🔜 Curve Finance
- 🔜 Balancer
- 🔜 Other AMM protocols

## Key Features

- **Real-time block data**: Aggregated swap activity per block
- **Chain-agnostic**: Works on any EVM-compatible blockchain
- **DEX-agnostic**: Universal output format across all DEX types
- **Stateless design**: No complex state management required
- **Raw volume data**: Volumes in token units for accurate calculations

## Development

Each DEX package is self-contained with:
- Cargo.toml configuration
- Substreams manifest (substreams.yaml)
- Test scripts
- Documentation

All implementations must:
1. Import the common proto definitions
2. Output data in `TickerOutput` format
3. Aggregate all swap activity per pool per block

See individual DEX folders for specific implementation details and testing instructions.

## Building

```bash
# Build all packages
cargo build --release

# Build WASM for a specific DEX
cd dexes/uniswap-v3-forks
cargo build --target wasm32-unknown-unknown --release
```

## Testing

Each DEX implementation includes a test script:
```bash
cd dexes/uniswap-v3-forks
./test.sh
```

## Contributing

When adding a new DEX:
1. Create a new folder under `dexes/`
2. Use the common proto format (`TickerOutput`)
3. Follow the existing implementation patterns
4. Include comprehensive documentation and tests