use crate::common::SwapAggregation;
use dex_common::{int256_to_bigint, uint160_to_bigint};
use std::collections::HashMap;
use substreams::scalar::BigInt;
use substreams_ethereum::block_view::LogView;

/// Process a V3 Swap event and update pool aggregations
/// Works for both Uniswap V3 and PancakeSwap V3 (ignores protocol fees)
pub fn process_swap_event(
    log: &LogView,
    pool_aggregations: &mut HashMap<Vec<u8>, SwapAggregation>,
) {
    // V3 Swap event structure:
    // - topics[0]: event signature
    // - topics[1]: indexed sender address
    // - topics[2]: indexed recipient address
    // - data: amount0 (int256), amount1 (int256), sqrtPriceX96 (uint160), liquidity (uint128), tick (int24)
    // Data layout: 32 + 32 + 32 + 32 + 32 = 160 bytes minimum
    // PancakeSwap V3 has 64 extra bytes (protocolFeesToken0, protocolFeesToken1) which we ignore
    if log.data().len() < 160 || log.topics().len() < 3 {
        return;
    }

    let pool_address = log.log.address.to_vec();
    let entry = pool_aggregations.entry(pool_address).or_default();

    // Parse amount0 (int256)
    let amount0_bytes = &log.data()[0..32];
    let amount0 = int256_to_bigint(amount0_bytes);

    // Parse amount1 (int256)
    let amount1_bytes = &log.data()[32..64];
    let amount1 = int256_to_bigint(amount1_bytes);

    // Calculate absolute volumes
    // Swap amounts are signed: negative = tokens out, positive = tokens in
    // We need absolute values since volume tracks total traded regardless of direction
    let abs_amount0 = if amount0 < BigInt::zero() {
        amount0.neg()
    } else {
        amount0
    };

    let abs_amount1 = if amount1 < BigInt::zero() {
        amount1.neg()
    } else {
        amount1
    };

    entry.volume_token0 = entry.volume_token0.clone() + abs_amount0;
    entry.volume_token1 = entry.volume_token1.clone() + abs_amount1;
    entry.swap_count += 1;

    // Parse sqrtPriceX96 (uint160) - bytes 64-96
    // This is the raw sqrtPriceX96 value that clients will use to calculate price
    let price_bytes = &log.data()[64..96];
    entry.last_sqrt_price = uint160_to_bigint(price_bytes);
}
