# Universal Protocol Buffers for DEX Data

This directory contains the universal protocol buffer definitions that ALL DEX implementations must use, ensuring complete consistency across different DEX types (V2, V3, Curve, Balancer, etc.).

## Files

### dex_common.proto
The universal output format that ALL DEX packages must use - whether they're V2 AMMs, V3 concentrated liquidity, Curve stable swaps, or any other DEX type.

Key messages:
- `TickerOutput`: Standard wrapper containing ticker data for any DEX
- `PoolTicker`: Universal format for aggregated trading data per pool per block (volumes, price, swap count)

## Usage

Each DEX package should:
1. Import this proto in their `substreams.yaml`
2. Output data in the `TickerOutput` format
3. Generate ticker data for each pool with activity

### Build Configuration

Each DEX package needs a `build.rs` file to compile the shared proto:

```rust
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Compile proto file
    prost_build::compile_protos(&["../../proto/dex_common.proto"], &["../../proto"])?;
    
    Ok(())
}
```

And include the generated code in `src/pb/mod.rs`:

```rust
pub mod dex {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/dex.common.v1.rs"));
        }
    }
}
```

## Benefits

- **Universal format**: Every DEX type (V2, V3, Curve, etc.) outputs identical structure
- **Easy aggregation**: Combine data from any DEX without transformation
- **Future-proof**: New DEX types automatically compatible with existing pipelines
- **Simplified processing**: One parser handles all DEX data
- **Consistent field naming**: Same fields regardless of underlying DEX mechanics