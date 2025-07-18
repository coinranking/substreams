# DEX 24-Hour Rolling Volume Substreams

A collection of Substreams packages for tracking 24-hour rolling volumes across various decentralized exchanges (DEXes).

## Project Structure

```
substreams/
├── proto/                 # Common protocol buffer definitions
│   └── dex_common.proto  # Shared output format for all DEXes
├── dexes/                # Individual DEX implementations
│   └── uniswap-v3/      # Uniswap V3 implementation
└── README.md            # This file
```

## Common Output Format

All DEX packages use the same output format defined in `proto/dex_common.proto`. This ensures consistent data structure across different DEXes, making it easy to:
- Aggregate volumes across multiple DEXes
- Build unified downstream consumers
- Add new DEXes without changing data pipelines

## Supported DEXes

- ✅ Uniswap V3 (Ethereum)
- 🔜 PancakeSwap V3 (BSC)
- 🔜 Sushiswap V2
- 🔜 Curve

## Features

- 24-hour rolling volume calculation
- 5-minute aggregation buckets
- Efficient circular buffer implementation (288 buckets)
- Support for partial data (no full history required)
- Chain-agnostic design

## Development

Each DEX package is self-contained with its own:
- Cargo.toml
- Test scripts
- Documentation
- Substreams configuration

See individual DEX folders for specific setup and testing instructions.