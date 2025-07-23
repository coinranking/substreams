# Common Protocol Buffers

This directory contains shared protocol buffer definitions used across all DEX implementations.

## Files

### dex_common.proto
The unified output format for all DEX packages. This ensures consistent data structure across different DEXes.

Key messages:
- `DexOutput`: Main output containing all data
- `DexInfo`: Identifies which DEX/chain/version
- `PoolCreated`: Pool/pair creation events
- `PoolTicker`: Aggregated trading data per pool per block (volumes, price, 24h volumes)

## Usage

Each DEX package should:
1. Import this proto in their `substreams.yaml`
2. Output data in the `DexOutput` format
3. Populate the `DexInfo` field appropriately

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

- Unified data format across all DEXes
- Easy aggregation of cross-DEX volumes
- Simplified downstream processing
- Consistent field naming and types