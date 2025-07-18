use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile both the common proto and the legacy output proto
    prost_build::compile_protos(
        &["../../proto/dex_common.proto", "proto/output.proto"], 
        &["../../proto", "proto/"]
    )?;

    Ok(())
}
