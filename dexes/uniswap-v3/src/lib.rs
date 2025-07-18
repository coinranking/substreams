mod pb;

use pb::uniswap::types::v1::events::pool_event;
use pb::uniswap::types::v1::{Events, Pools};
use pb::uniswap::v3::mvp::{PoolCreated, RollingVolumeData, SwapEvent, TokenInfo, UniswapV3Output};
use std::collections::HashMap;
use substreams::prelude::StoreGetString;
use substreams::store::{
    StoreGet, StoreGetBigDecimal, StoreGetBigInt, StoreGetInt64, StoreMax, StoreMaxInt64, StoreNew,
    StoreSet, StoreSetString,
};
use substreams_ethereum::pb::eth::v2 as eth;

struct VolumeAccumulator {
    token0_volume: String,
    token1_volume: String,
}

impl VolumeAccumulator {
    fn new() -> Self {
        Self {
            token0_volume: "0".to_string(),
            token1_volume: "0".to_string(),
        }
    }

    fn add_swap(&mut self, amount0: &str, amount1: &str) {
        // Convert to absolute values and add
        let abs_amount0 = amount0.trim_start_matches('-');
        let abs_amount1 = amount1.trim_start_matches('-');

        self.token0_volume = add_strings(&self.token0_volume, abs_amount0);
        self.token1_volume = add_strings(&self.token1_volume, abs_amount1);
    }
}

// Simple string addition for demo - in production use proper BigDecimal
fn add_strings(a: &str, b: &str) -> String {
    (a.parse::<f64>().unwrap_or(0.0) + b.parse::<f64>().unwrap_or(0.0)).to_string()
}

/// Main map handler that processes Uniswap V3 events and outputs pool creations,
/// swaps, and 24-hour rolling volumes.
#[substreams::handlers::map]
fn map_uniswap_output(
    block: eth::Block,
    pools_created: Pools,
    events: Events,
    _swaps_volume: StoreGetBigDecimal,
    _total_tx_counts: StoreGetBigInt,
    completed_periods: StoreGetInt64,
    _minute_volumes: StoreGetString,
    rolling_volumes: StoreGetString,
) -> Result<UniswapV3Output, substreams::errors::Error> {
    let mut output = UniswapV3Output {
        pools_created: vec![],
        tokens: vec![],
        swaps: vec![],
        rolling_volumes: vec![],
    };

    // Process actual Uniswap pool creation events
    for pool in pools_created.pools {
        let pool_created = PoolCreated {
            pool_address: pool.address.clone(),
            token0: pool
                .token0
                .as_ref()
                .map(|t| t.address.clone())
                .unwrap_or_default(),
            token1: pool
                .token1
                .as_ref()
                .map(|t| t.address.clone())
                .unwrap_or_default(),
            fee: pool.fee_tier.parse::<u32>().unwrap_or_default(),
            block_number: block.number,
            transaction_hash: pool.transaction_id.clone(),
        };

        output.pools_created.push(pool_created);

        // Add token information if available
        if let Some(token0) = &pool.token0 {
            let token_info = TokenInfo {
                address: token0.address.clone(),
                symbol: token0.symbol.clone(),
                name: token0.name.clone(),
                decimals: token0.decimals as u32,
            };
            output.tokens.push(token_info);
        }

        if let Some(token1) = &pool.token1 {
            let token_info = TokenInfo {
                address: token1.address.clone(),
                symbol: token1.symbol.clone(),
                name: token1.name.clone(),
                decimals: token1.decimals as u32,
            };
            output.tokens.push(token_info);
        }
    }

    // Process swap events
    for pool_event in &events.pool_events {
        if let Some(pool_event::Type::Swap(swap)) = &pool_event.r#type {
            let swap_event = SwapEvent {
                pool_address: pool_event.pool_address.clone(),
                sender: swap.sender.clone(),
                recipient: swap.recipient.clone(),
                amount0: swap.amount_0.clone(),
                amount1: swap.amount_1.clone(),
                sqrt_price: swap.sqrt_price.clone(),
                liquidity: swap.liquidity.clone(),
                tick: swap.tick.parse::<i32>().unwrap_or_default(),
                block_number: block.number,
                transaction_hash: pool_event.transaction_id.clone(),
                timestamp: pool_event.timestamp,
                log_ordinal: pool_event.log_ordinal as u32,
            };

            output.swaps.push(swap_event);
        }
    }

    // Accumulate volumes per pool for this block
    let mut pool_volumes: HashMap<String, VolumeAccumulator> = HashMap::new();

    for swap in &output.swaps {
        let accumulator = pool_volumes
            .entry(swap.pool_address.clone())
            .or_insert_with(VolumeAccumulator::new);
        accumulator.add_swap(&swap.amount0, &swap.amount1);
    }

    // Get rolling volumes for all pools that have them
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);

    // Check all pools with swaps for rolling volume data
    let mut checked_pools = std::collections::HashSet::new();

    for pool_event in &events.pool_events {
        if let Some(pool_event::Type::Swap(_)) = &pool_event.r#type {
            let pool_address = &pool_event.pool_address;

            if checked_pools.insert(pool_address.clone()) {
                let rolling_key = format!("{pool_address}:rolling");
                let current_rolling = rolling_volumes
                    .get_last(&rolling_key)
                    .unwrap_or_else(|| "0,0".to_string());

                let parts: Vec<&str> = current_rolling.split(',').collect();
                let token0_vol = parts.first().unwrap_or(&"0");
                let token1_vol = parts.get(1).unwrap_or(&"0");

                // Log for debugging
                substreams::log::debug!(
                    "Pool {}: rolling volume token0={}, token1={}",
                    pool_address,
                    token0_vol,
                    token1_vol
                );

                // Always show rolling volume data for pools with swaps
                let last_completed = completed_periods.get_last(pool_address).unwrap_or(0) as u64;
                let current_period = (block_timestamp / 300) as u64;

                output.rolling_volumes.push(RollingVolumeData {
                    pool_address: pool_address.clone(),
                    token0_volume_24h: token0_vol.to_string(),
                    token1_volume_24h: token1_vol.to_string(),
                    last_update_timestamp: block_timestamp as u64,
                    last_completed_period: last_completed,
                    bucket_count: if last_completed > 0 {
                        std::cmp::min(
                            288,
                            current_period.saturating_sub(
                                current_period.saturating_sub(std::cmp::min(current_period, 288)),
                            ),
                        ) as u32
                    } else {
                        0
                    },
                });
            }
        }
    }

    substreams::log::info!(
        "Block {}: {} pools created, {} swaps, {} rolling volumes",
        block.number,
        output.pools_created.len(),
        output.swaps.len(),
        output.rolling_volumes.len()
    );

    Ok(output)
}

/// Store handler that tracks the last completed 5-minute period for each pool.
/// Uses max update policy to ensure we don't go backwards in time.
#[substreams::handlers::store]
fn store_completed_periods(block: eth::Block, events: Events, output: StoreMaxInt64) {
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);
    let current_period = (block_timestamp / 300) as i64; // 300 seconds = 5 minutes

    // Track which pools had swaps in this block
    let mut pools_with_swaps = std::collections::HashSet::new();

    for pool_event in events.pool_events {
        if let Some(pool_event::Type::Swap(_)) = pool_event.r#type {
            pools_with_swaps.insert(pool_event.pool_address.clone());
        }
    }

    // Update last completed 5-minute period for pools with activity
    // Using max policy ensures we don't go backwards
    for pool_address in pools_with_swaps {
        output.max(0, &pool_address, current_period);
    }
}

/// Store handler that accumulates swap volumes into 5-minute buckets.
/// Each pool has 288 buckets for a 24-hour circular buffer.
#[substreams::handlers::store]
fn store_minute_volumes(
    block: eth::Block,
    events: Events,
    _completed_periods: StoreGetInt64,
    store: StoreSetString,
) {
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);
    let current_period = block_timestamp / 300; // 5-minute periods
    let _bucket_index = current_period % 288;

    // Accumulate all swaps in this block by pool
    let mut pool_volumes: HashMap<String, VolumeAccumulator> = HashMap::new();

    for pool_event in &events.pool_events {
        if let Some(pool_event::Type::Swap(swap)) = &pool_event.r#type {
            let accumulator = pool_volumes
                .entry(pool_event.pool_address.clone())
                .or_insert_with(VolumeAccumulator::new);
            accumulator.add_swap(&swap.amount_0, &swap.amount_1);
        }
    }

    // Store volumes in the current period's bucket
    // Since we can't read our own state, we'll overwrite the bucket
    // This means each bucket holds the volume for ONE 5-minute period
    for (pool_address, accumulator) in pool_volumes {
        let bucket_idx = current_period % 288;
        let bucket_key = format!("{pool_address}:bucket:{bucket_idx}");
        let bucket_value = format!(
            "{},{}",
            accumulator.token0_volume, accumulator.token1_volume
        );

        substreams::log::debug!(
            "Storing volume for pool {} in bucket {} (period {}): {}",
            pool_address,
            bucket_idx,
            current_period,
            bucket_value
        );

        store.set(0, &bucket_key, &bucket_value);

        // Store which period this bucket represents
        let period_key = format!("{pool_address}:bucket:{bucket_idx}:period");
        store.set(0, &period_key, &current_period.to_string());
    }
}

/// Store handler that maintains 24-hour rolling volumes for each pool.
/// Updates when 5-minute period boundaries are crossed by summing all bucket values.
#[substreams::handlers::store]
fn store_rolling_volumes(
    block: eth::Block,
    events: Events,
    completed_periods: StoreGetInt64,
    minute_volumes: StoreGetString,
    store: StoreSetString,
) {
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);
    let current_period = block_timestamp / 300; // 5-minute periods

    // Track pools with activity to update their rolling volumes
    let mut active_pools = std::collections::HashSet::new();

    for pool_event in &events.pool_events {
        if let Some(pool_event::Type::Swap(_)) = &pool_event.r#type {
            active_pools.insert(pool_event.pool_address.clone());
        }
    }

    // Update rolling volumes for active pools
    for pool_address in active_pools {
        let _last_completed = completed_periods.get_last(&pool_address).unwrap_or(0);

        // Always recalculate rolling volume from all buckets
        let rolling_key = format!("{pool_address}:rolling");

        // Sum all valid buckets for 24-hour volume
        let mut total_token0 = "0".to_string();
        let mut total_token1 = "0".to_string();

        // Check each bucket to see if it's within the last 24 hours
        for bucket_idx in 0..288 {
            let bucket_key = format!("{pool_address}:bucket:{bucket_idx}");
            let period_key = format!("{pool_address}:bucket:{bucket_idx}:period");

            // Get the period when this bucket was last updated
            if let Some(bucket_period_str) = minute_volumes.get_last(&period_key) {
                if let Ok(bucket_period) = bucket_period_str.parse::<u64>() {
                    // Only include if within last 24 hours (288 periods)
                    if current_period as u64 >= bucket_period
                        && (current_period as u64).saturating_sub(bucket_period) < 288
                    {
                        let bucket_value = minute_volumes
                            .get_last(&bucket_key)
                            .unwrap_or_else(|| "0,0".to_string());
                        let parts: Vec<&str> = bucket_value.split(',').collect();

                        total_token0 = add_strings(&total_token0, parts.first().unwrap_or(&"0"));
                        total_token1 = add_strings(&total_token1, parts.get(1).unwrap_or(&"0"));
                    }
                }
            }
        }

        let new_rolling_value = format!("{total_token0},{total_token1}");
        substreams::log::debug!(
            "Updating rolling volume for pool {}: {}",
            pool_address,
            new_rolling_value
        );
        store.set(0, &rolling_key, &new_rolling_value);
    }
}
