// Uniswap V3 and Compatible Forks Substreams Implementation
//
// This implementation works with all Uniswap V3 forks across different blockchains,
// including:
// - Uniswap V3 (Ethereum, Polygon, Arbitrum, Optimism, Base, etc.)
// - QuickSwap V3 / Algebra Protocol (Polygon)
// - PancakeSwap V3 (BSC, Ethereum)
// - And other V3 forks that maintain the same event signatures
//
// IMPORTANT: Output Format
// ------------------------
// 1. Volumes are reported in RAW TOKEN UNITS (not decimal-adjusted)
//    Example: 500 USDC (6 decimals) is reported as "500000000"
//
// 2. Price is reported as sqrtPriceX96 (the raw value from swap events)
//    Clients must calculate actual price using:
//    price = (sqrtPriceX96 / 2^96)^2 * 10^(token0_decimals - token1_decimals)

mod pb;

use crate::pb::dex::common::v1::{PoolTicker, TickerOutput};
use dex_common::{ensure_0x_prefix, format_bigint, int256_to_bigint, uint160_to_bigint};
use std::collections::HashMap;
use substreams::scalar::BigInt;
use substreams::Hex;
use substreams_ethereum::block_view::LogView;
use substreams_ethereum::pb::eth::v2 as eth;

// Event signatures (keccak256 hashes)
// Swap(address,address,int256,int256,uint160,uint128,int24) - with indexed sender/recipient
const SWAP_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67");

// Aggregation struct for pool data
#[derive(Clone)]
struct SwapAggregation {
    volume_token0: BigInt,
    volume_token1: BigInt,
    swap_count: u32,
    last_sqrt_price: BigInt,
}

impl Default for SwapAggregation {
    fn default() -> Self {
        Self {
            volume_token0: BigInt::zero(),
            volume_token1: BigInt::zero(),
            swap_count: 0,
            last_sqrt_price: BigInt::zero(),
        }
    }
}

/// Process a swap event and update pool aggregations
fn process_swap_event(log: &LogView, pool_aggregations: &mut HashMap<Vec<u8>, SwapAggregation>) {
    // Swap event structure:
    // - topics[0]: event signature
    // - topics[1]: indexed sender address
    // - topics[2]: indexed recipient address
    // - data: amount0 (int256), amount1 (int256), sqrtPriceX96 (uint160), liquidity (uint128), tick (int24)
    // Data layout: 32 + 32 + 32 + 32 + 32 = 160 bytes minimum
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

#[substreams::handlers::map]
pub fn map_v3_ticker_output(block: eth::Block) -> Result<TickerOutput, substreams::errors::Error> {
    let mut pool_aggregations: HashMap<Vec<u8>, SwapAggregation> = HashMap::new();

    // Process all swap events
    for log in block.logs() {
        // Early exit if no topics
        if log.topics().is_empty() {
            continue;
        }

        // Direct byte comparison - no string allocation needed
        if log.topics()[0] == SWAP_EVENT_SIG {
            process_swap_event(&log, &mut pool_aggregations);
        }
    }

    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds as u64)
        .ok_or_else(|| {
            substreams::errors::Error::msg(format!(
                "Block {} missing header or timestamp",
                block.number
            ))
        })?;

    // Create output with ticker data
    let mut tickers = vec![];

    for (pool_address_bytes, aggregation) in pool_aggregations {
        let pool_address = ensure_0x_prefix(&Hex(&pool_address_bytes).to_string());

        tickers.push(PoolTicker {
            pool_address,
            block_volume_token0: format_bigint(&aggregation.volume_token0),
            block_volume_token1: format_bigint(&aggregation.volume_token1),
            swap_count: aggregation.swap_count,
            sqrt_price_x96: format_bigint(&aggregation.last_sqrt_price),
            block_number: block.number,
            timestamp: timestamp_seconds,
        });
    }

    Ok(TickerOutput { tickers })
}
