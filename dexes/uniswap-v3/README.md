# Uniswap V3 24-Hour Rolling Volume Substream

This Substreams package provides real-time 24-hour rolling volume tracking for Uniswap V3 pools on Ethereum mainnet.

## Features

- Tracks pool creation events
- Captures all swap transactions
- Calculates 24-hour rolling volumes using a circular buffer approach
- Uses 5-minute aggregation periods for efficient storage
- Prevents partial period data from affecting volume calculations

## Architecture

The system uses three store modules:
- `store_completed_periods`: Tracks the last completed 5-minute period for each pool
- `store_minute_volumes`: Stores volume data in 5-minute buckets (288 buckets for 24 hours)
- `store_rolling_volumes`: Maintains the current 24-hour rolling volume for each pool

## Testing

Use the test script with various options:

```bash
# Run with default settings
./test.sh

# Test with swap activity and filtering
./test.sh -s 12400000 -e 12400010 -f

# Use UI output format
./test.sh --output ui

# Show help
./test.sh -h
```

## Building

```bash
# Build the WASM module
cargo build --target wasm32-unknown-unknown --release

# Generate required protobuf files from imported packages
substreams protogen substreams.yaml --output-path src/generated
```

## Packaging for Deployment

To prepare your Substreams for deployment:

```bash
# 1. Build the WASM module
cargo build --target wasm32-unknown-unknown --release

# 2. Generate required protobuf files (if not already present)
substreams protogen substreams.yaml --output-path src/generated

# 3. Create the SPKG package
substreams pack
# This creates: coinranking-uniswap-v3-v0.1.0.spkg
```

The SPKG file contains:
- Compiled WASM module
- Protobuf definitions  
- Substreams manifest
- All dependencies

## Running the Package

To test your packaged Substreams:

```bash
# Load JWT token from .env file
source ../../.env

# For testing: Run 100 blocks from a specific start
substreams run coinranking-uniswap-v3-v0.1.0.spkg map_uniswap_ticker_output \
  --substreams-api-token="$SUBSTREAMS_API_TOKEN" \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  -s 22964000 \    # Start at specific block (overrides initialBlock in yaml)
  -t +100          # Stop after 100 blocks

# For production: Start from 24+ hours ago for complete rolling volumes
substreams run coinranking-uniswap-v3-v0.1.0.spkg map_uniswap_ticker_output \
  --substreams-api-token="$SUBSTREAMS_API_TOKEN" \
  --substreams-endpoint="mainnet.eth.streamingfast.io:443" \
  -s -7500 \       # ~25 hours ago (7200 blocks/day + margin)
  -t 0             # Stream indefinitely
```

### Understanding Block Numbers

- **Ethereum block time**: ~12 seconds
- **Blocks per day**: ~7,200 (86,400 seconds ÷ 12)
- **Recommended start**: `-7500` (25 hours ago)
  - Provides margin for block time variance
  - Ensures complete 24h rolling volume data
  - Allows proper warm-up of volume calculations

The `.env` file should contain:
```
SUBSTREAMS_API_TOKEN=your-jwt-token-here
```

## Publishing

Once tested, you can publish your package:

```bash
# Publish to the official Substreams.dev registry
substreams publish coinranking-uniswap-v3-v0.1.0.spkg
```

This makes your package discoverable on the Substreams registry where others can easily find and use it.

## Output Format

The main output includes:
- **pools_created**: New Uniswap V3 pools
- **tokens**: Token information for pools
- **swaps**: Individual swap events
- **rolling_volumes**: 24-hour rolling volumes per pool

## Development

### Generated Files

The `src/generated/` directory contains protobuf-generated Rust code from the imported Uniswap substreams package. These files are:
- Not tracked in git (see `.gitignore`)
- Required for compilation but should not be edited
- Generated using the `substreams protogen` command

To regenerate these files:
```bash
# Generate Rust bindings from the imported Uniswap package
substreams protogen substreams.yaml --output-path src/generated

# Note: Only uniswap.types.v1.rs is actually used by our code
```

This command extracts and generates Rust code from the imported Uniswap SPKG defined in `substreams.yaml`.

### Code Quality

This project uses Rust's standard formatting and linting tools:

```bash
# Format code
cargo fmt

# Check for common mistakes and improvements
cargo clippy

# Run before committing
cargo fmt && cargo clippy && cargo build --target wasm32-unknown-unknown --release
```

### Contributing

1. Always run `cargo fmt` before committing
2. Fix all clippy warnings (except those in generated code)
3. Ensure the WASM build succeeds
4. Update tests if modifying functionality