use crate::pb::evm::common::v1::NewPool;
use dex_common::ensure_0x_prefix;
use substreams::Hex;
use substreams_ethereum::block_view::LogView;

/// Process V2 PairCreated event
/// Event: PairCreated(address indexed token0, address indexed token1, address pair, uint)
/// Topics: [sig, token0, token1]
/// Data: [pair_address (32 bytes), pair_count (32 bytes)]
pub fn process_v2_pair_created(
    log: &LogView,
    block_number: u64,
    timestamp: u64,
) -> Option<NewPool> {
    // V2 PairCreated has 3 topics and at least 32 bytes of data for pair address
    if log.topics().len() < 3 || log.data().len() < 32 {
        return None;
    }

    // Token addresses are in topics 1 and 2 (right-padded to 32 bytes, address is last 20 bytes)
    let token0_bytes = &log.topics()[1].as_slice()[12..32];
    let token1_bytes = &log.topics()[2].as_slice()[12..32];

    // Pool address is in data bytes 0-32 (right-padded, address is last 20 bytes)
    let pool_address_bytes = &log.data()[12..32];

    // Factory is the log emitter
    let factory_address = ensure_0x_prefix(&Hex(&log.log.address).to_string());
    let pool_address = ensure_0x_prefix(&Hex(pool_address_bytes).to_string());
    let token0_address = ensure_0x_prefix(&Hex(token0_bytes).to_string());
    let token1_address = ensure_0x_prefix(&Hex(token1_bytes).to_string());

    Some(NewPool {
        pool_address,
        factory_address,
        token0_address,
        token1_address,
        fee: 0, // V2 pools don't have variable fees
        block_number,
        timestamp,
    })
}

/// Process V3 PoolCreated event
/// Event: PoolCreated(address indexed token0, address indexed token1, uint24 indexed fee, int24 tickSpacing, address pool)
/// Topics: [sig, token0, token1, fee]
/// Data: [tickSpacing (32 bytes), pool_address (32 bytes)]
pub fn process_v3_pool_created(
    log: &LogView,
    block_number: u64,
    timestamp: u64,
) -> Option<NewPool> {
    // V3 PoolCreated has 4 topics and at least 64 bytes of data
    if log.topics().len() < 4 || log.data().len() < 64 {
        return None;
    }

    // Token addresses are in topics 1 and 2 (right-padded to 32 bytes, address is last 20 bytes)
    let token0_bytes = &log.topics()[1].as_slice()[12..32];
    let token1_bytes = &log.topics()[2].as_slice()[12..32];

    // Fee is in topic 3 (uint24, stored in last 3 bytes of 32-byte word)
    let fee_bytes = &log.topics()[3].as_slice()[29..32];
    let fee = ((fee_bytes[0] as u32) << 16) | ((fee_bytes[1] as u32) << 8) | (fee_bytes[2] as u32);

    // Pool address is in data bytes 32-64 (tickSpacing is 0-32)
    let pool_address_bytes = &log.data()[44..64];

    // Factory is the log emitter
    let factory_address = ensure_0x_prefix(&Hex(&log.log.address).to_string());
    let pool_address = ensure_0x_prefix(&Hex(pool_address_bytes).to_string());
    let token0_address = ensure_0x_prefix(&Hex(token0_bytes).to_string());
    let token1_address = ensure_0x_prefix(&Hex(token1_bytes).to_string());

    Some(NewPool {
        pool_address,
        factory_address,
        token0_address,
        token1_address,
        fee,
        block_number,
        timestamp,
    })
}
