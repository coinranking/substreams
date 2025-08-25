use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile proto file
    prost_build::compile_protos(&["../../proto/dex_common.proto"], &["../../proto"])?;

    Ok(())
}
