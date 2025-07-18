# Common Protocol Buffers

This directory contains shared protocol buffer definitions used across all DEX implementations.

## Files

### dex_common.proto
The unified output format for all DEX packages. This ensures consistent data structure across different DEXes.

Key messages:
- `DexOutput`: Main output containing all data
- `DexInfo`: Identifies which DEX/chain/version
- `PoolCreated`: Pool/pair creation events
- `SwapEvent`: Individual trades
- `RollingVolumeData`: 24-hour volume aggregates

## Usage

Each DEX package should:
1. Import this proto in their `substreams.yaml`
2. Output data in the `DexOutput` format
3. Populate the `DexInfo` field appropriately

## Benefits

- Unified data format across all DEXes
- Easy aggregation of cross-DEX volumes
- Simplified downstream processing
- Consistent field naming and types