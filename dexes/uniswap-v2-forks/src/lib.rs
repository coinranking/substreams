// Uniswap V2 and Compatible Forks Substreams Implementation
//
// This implementation works with all Uniswap V2 forks across different blockchains,
// including:
// - Uniswap V2 (Ethereum)
// - SushiSwap (Ethereum, Polygon, Arbitrum, etc.)
// - PancakeSwap V2 (BSC)
// - And other V2 forks that maintain the same event signatures
//
// IMPORTANT: Output Format
// ------------------------
// 1. Volumes are reported in RAW TOKEN UNITS (not decimal-adjusted)
//    Example: 500 USDC (6 decimals) is reported as "500000000"
//
// 2. Price is reported as sqrtPriceX96 (converted from V2 reserves)
//    Clients must calculate actual price using:
//    price = (sqrtPriceX96 / 2^96)^2 * 10^(token0_decimals - token1_decimals)
//
// 3. This matches V3's output format for consistency

mod pb;

use crate::pb::dex::common::v1::{PoolTicker, TickerOutput};
use dex_common::{
    calculate_sqrt_price_x96, ensure_0x_prefix, format_bigint, uint112_to_bigint, uint256_to_bigint,
};
use std::collections::HashMap;
use substreams::scalar::BigInt;
use substreams::Hex;
use substreams_ethereum::block_view::LogView;
use substreams_ethereum::pb::eth::v2 as eth;

// Event signatures (keccak256 hashes)
// Swap(address indexed sender, uint amount0In, uint amount1In, uint amount0Out, uint amount1Out, address indexed to)
const SWAP_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822");

// Sync(uint112 reserve0, uint112 reserve1)
const SYNC_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1");

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

/// Process a sync event and update pool aggregations
fn process_sync_event(log: &LogView, pool_aggregations: &mut HashMap<Vec<u8>, SwapAggregation>) {
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

#[substreams::handlers::map]
pub fn map_v2_ticker_output(block: eth::Block) -> Result<TickerOutput, substreams::errors::Error> {
    let mut pool_aggregations: HashMap<Vec<u8>, SwapAggregation> = HashMap::new();

    // Process all swap and sync events
    for log in block.logs() {
        // Early exit if no topics
        if log.topics().is_empty() {
            continue;
        }

        // Direct byte comparison - no string allocation needed
        if log.topics()[0] == SWAP_EVENT_SIG {
            process_swap_event(&log, &mut pool_aggregations);
        } else if log.topics()[0] == SYNC_EVENT_SIG {
            process_sync_event(&log, &mut pool_aggregations);
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
