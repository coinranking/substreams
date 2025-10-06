// Universal DEX Substreams Implementation
//
// This implementation works with multiple DEX protocols across different blockchains:
// - Uniswap V2 + forks (SushiSwap, PancakeSwap V2, QuickSwap V2, etc.)
// - Uniswap V3
// - PancakeSwap V3
//
// IMPORTANT: Output Format
// ------------------------
// 1. Volumes are reported in RAW TOKEN UNITS (not decimal-adjusted)
//    Example: 500 USDC (6 decimals) is reported as "500000000"
//
// 2. Price is reported as sqrtPriceX96
//    - V3: Directly from swap events
//    - V2: Calculated from reserves via Sync events as sqrt(reserve1/reserve0) * 2^96
//    Clients must calculate actual price using:
//    price = (sqrtPriceX96 / 2^96)^2 * 10^(token0_decimals - token1_decimals)

mod common;
mod pb;
mod v2;
mod v3;

use crate::common::SwapAggregation;
use crate::pb::dex::common::v1::{PoolTicker, TickerOutput};
use dex_common::{ensure_0x_prefix, format_bigint};
use std::collections::HashMap;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;

// Event signatures (keccak256 hashes)

// V2 events (Uniswap V2, SushiSwap, PancakeSwap V2, QuickSwap V2, etc.)
const V2_SWAP_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822");
const V2_SYNC_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1");

// V3 events
const UNISWAP_V3_SWAP_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67");
const PANCAKESWAP_V3_SWAP_EVENT_SIG: [u8; 32] =
    hex_literal::hex!("19b47279256b2a23a1665c810c8d55a1758940ee09377d4f8d26497a3577dc83");

#[substreams::handlers::map]
pub fn map_dex_ticker_output(block: eth::Block) -> Result<TickerOutput, substreams::errors::Error> {
    let mut pool_aggregations: HashMap<Vec<u8>, SwapAggregation> = HashMap::new();

    // Process all DEX events
    for log in block.logs() {
        // Early exit if no topics
        if log.topics().is_empty() {
            continue;
        }

        // Route to appropriate processor based on event signature
        match log.topics()[0].as_slice() {
            // V2 events
            topic if topic == V2_SWAP_EVENT_SIG => {
                v2::process_swap_event(&log, &mut pool_aggregations)
            }
            topic if topic == V2_SYNC_EVENT_SIG => {
                v2::process_sync_event(&log, &mut pool_aggregations)
            }

            // V3 events (both Uniswap and PancakeSwap)
            topic
                if topic == UNISWAP_V3_SWAP_EVENT_SIG || topic == PANCAKESWAP_V3_SWAP_EVENT_SIG =>
            {
                v3::process_swap_event(&log, &mut pool_aggregations)
            }

            _ => {}
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
