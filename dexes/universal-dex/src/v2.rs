use crate::common::SwapAggregation;
use dex_common::{calculate_sqrt_price_x96, uint112_to_bigint, uint256_to_bigint};
use std::collections::HashMap;
use substreams_ethereum::block_view::LogView;

/// Process a V2 Swap event and update pool aggregations
pub fn process_swap_event(
    log: &LogView,
    pool_aggregations: &mut HashMap<Vec<u8>, SwapAggregation>,
) {
    // V2 Swap event structure:
    // - topics[0]: event signature
    // - topics[1]: indexed sender address
    // - topics[2]: indexed to address
    // - data: amount0In (uint256), amount1In (uint256), amount0Out (uint256), amount1Out (uint256)
    // Data layout: 32 + 32 + 32 + 32 = 128 bytes
    if log.data().len() < 128 || log.topics().len() < 3 {
        return;
    }

    let pool_address = log.log.address.to_vec();
    let entry = pool_aggregations.entry(pool_address).or_default();

    // Parse amount0In (uint256)
    let amount0_in_bytes = &log.data()[0..32];
    let amount0_in = uint256_to_bigint(amount0_in_bytes);

    // Parse amount1In (uint256)
    let amount1_in_bytes = &log.data()[32..64];
    let amount1_in = uint256_to_bigint(amount1_in_bytes);

    // Parse amount0Out (uint256)
    let amount0_out_bytes = &log.data()[64..96];
    let amount0_out = uint256_to_bigint(amount0_out_bytes);

    // Parse amount1Out (uint256)
    let amount1_out_bytes = &log.data()[96..128];
    let amount1_out = uint256_to_bigint(amount1_out_bytes);

    // Calculate volumes
    // For V2, volume is the sum of in and out amounts (one will be 0 for each direction)
    entry.volume_token0 = entry.volume_token0.clone() + amount0_in + amount0_out;
    entry.volume_token1 = entry.volume_token1.clone() + amount1_in + amount1_out;
    entry.swap_count += 1;
}

/// Process a V2 Sync event and update pool aggregations
pub fn process_sync_event(
    log: &LogView,
    pool_aggregations: &mut HashMap<Vec<u8>, SwapAggregation>,
) {
    // V2 Sync event structure:
    // - topics[0]: event signature
    // - data: reserve0 (uint112), reserve1 (uint112)
    // Data layout: 32 + 32 = 64 bytes (uint112 values are stored in 32-byte words)
    if log.data().len() < 64 {
        return;
    }

    let pool_address = log.log.address.to_vec();
    let entry = pool_aggregations.entry(pool_address).or_default();

    // Parse reserve0 (uint112 in 32-byte word)
    let reserve0_bytes = &log.data()[0..32];
    let reserve0 = uint112_to_bigint(reserve0_bytes);

    // Parse reserve1 (uint112 in 32-byte word)
    let reserve1_bytes = &log.data()[32..64];
    let reserve1 = uint112_to_bigint(reserve1_bytes);

    // Calculate sqrtPriceX96 from reserves to match V3 output format
    // sqrtPriceX96 = sqrt(reserve1 / reserve0) * 2^96
    entry.last_sqrt_price = calculate_sqrt_price_x96(&reserve0, &reserve1);
}
