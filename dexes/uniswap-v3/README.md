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
cargo build --target wasm32-unknown-unknown --release
```

## Deployment

```bash
substreams pack -o uniswap-v3-mvp.spkg
```

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
- Automatically included when downloading the Uniswap dependency

If you need to regenerate these files:
1. They come from the imported Uniswap substreams package defined in `substreams.yaml`
2. The files are generated during the Uniswap package build process
3. Only `uniswap.types.v1.rs` is actually used by our code

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