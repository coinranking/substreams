// ─────────────────────────────────────────────────────────────────────────────
// Uniswap V3 — stateless block-aggregated swap data
// ─────────────────────────────────────────────────────────────────────────────

mod pb;

use crate::pb::dex::common::v1::{DexOutput, PoolCreated, PoolTicker};
use crate::pb::uniswap::types::v1::events::pool_event;
use crate::pb::uniswap::types::v1::{Events, Pools};
use std::collections::HashMap;
use std::str::FromStr;
use substreams::scalar::BigDecimal;
use substreams_ethereum::pb::eth::v2 as eth;

/// Map handler that generates the final DexOutput with ticker information
/// Aggregates swap data per pool per block (stateless)
#[substreams::handlers::map]
pub fn map_uniswap_ticker_output(
    block: eth::Block,
    pools: Pools,
    events: Events,
) -> Result<DexOutput, substreams::errors::Error> {
    let mut dex_output = DexOutput {
        pools_created: vec![],
        tickers: vec![],
    };

    // Pass through pool creation events
    for pool in pools.pools {
        dex_output.pools_created.push(PoolCreated {
            pool_address: ensure_0x_prefix(&pool.address),
            token0: pool
                .token0
                .as_ref()
                .map(|token| ensure_0x_prefix(&token.address))
                .unwrap_or_default(),
            token1: pool
                .token1
                .as_ref()
                .map(|token| ensure_0x_prefix(&token.address))
                .unwrap_or_default(),
            fee: pool.fee_tier.parse::<u32>().unwrap_or_default(),
            block_number: block.number,
            transaction_hash: ensure_0x_prefix(&pool.transaction_id),
            token0_decimals: pool
                .token0
                .as_ref()
                .map(|token| token.decimals as u32)
                .unwrap_or(0),
            token1_decimals: pool
                .token1
                .as_ref()
                .map(|token| token.decimals as u32)
                .unwrap_or(0),
        });
    }

    // Aggregate current block's swap volumes and track last prices
    let mut pool_aggregations: HashMap<String, (BigDecimal, BigDecimal, u32, String)> =
        HashMap::new();

    for event in events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = event.r#type {
            let entry = pool_aggregations
                .entry(event.pool_address.clone())
                .or_insert((BigDecimal::zero(), BigDecimal::zero(), 0, String::new()));

            // Accumulate token0 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                entry.0 = entry.0.clone() + volume;
            }

            // Accumulate token1 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                entry.1 = entry.1.clone() + volume;
            }

            entry.2 += 1; // Increment swap count

            // Update the last sqrt_price for this pool (closing price)
            entry.3 = swap_event.sqrt_price.clone();
        }
    }

    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;

    // Generate ticker data for each pool that had swaps
    for (pool_address, (current_volume_token0, current_volume_token1, swaps, last_sqrt_price)) in
        pool_aggregations
    {
        dex_output.tickers.push(PoolTicker {
            pool_address: ensure_0x_prefix(&pool_address),
            block_volume_token0: format_bigdecimal(&current_volume_token0),
            block_volume_token1: format_bigdecimal(&current_volume_token1),
            swap_count: swaps,
            close_price: calculate_price_from_sqrt(&last_sqrt_price),
            block_number: block.number,
            timestamp: timestamp_seconds,
        });
    }

    Ok(dex_output)
}

/// Ensure address has 0x prefix
fn ensure_0x_prefix(address: &str) -> String {
    if address.starts_with("0x") || address.starts_with("0X") {
        address.to_string()
    } else {
        format!("0x{address}")
    }
}

/// Calculate price from Uniswap V3 sqrtPriceX96
/// sqrtPriceX96 represents sqrt(price) * 2^96
/// Returns the price as token1/token0
fn calculate_price_from_sqrt(sqrt_price_str: &str) -> String {
    if sqrt_price_str.is_empty() || sqrt_price_str == "0" {
        return "0".to_string();
    }

    match BigDecimal::from_str(sqrt_price_str) {
        Ok(sqrt_price_x96) => {
            // Calculate price = (sqrtPriceX96 / 2^96)^2
            let two_96 = BigDecimal::from_str("79228162514264337593543950336").unwrap(); // 2^96
            let sqrt_price = sqrt_price_x96 / two_96;
            let price = sqrt_price.clone() * sqrt_price;
            format_bigdecimal(&price)
        }
        Err(_) => "0".to_string(),
    }
}

/// Format a BigDecimal to a string with at most 18 decimal places
/// Removes trailing zeros and decimal point if unnecessary
#[inline]
fn format_bigdecimal(big_decimal: &BigDecimal) -> String {
    let mut decimal_string = big_decimal.to_string();

    if let Some(decimal_point_index) = decimal_string.find('.') {
        // Truncate to maximum 18 decimal places
        let truncate_position = usize::min(decimal_point_index + 1 + 18, decimal_string.len());
        decimal_string.truncate(truncate_position);

        // Remove trailing zeros
        while decimal_string.ends_with('0') {
            decimal_string.pop();
        }

        // Remove decimal point if no decimals remain
        if decimal_string.ends_with('.') {
            decimal_string.pop();
        }
    }

    if decimal_string.is_empty() {
        "0".into()
    } else {
        decimal_string
    }
}
