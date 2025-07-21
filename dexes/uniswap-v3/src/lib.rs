mod pb;

use pb::dex::common::v1::{DexInfo, DexOutput, PoolCreated, PoolTicker};
use pb::uniswap::types::v1::events::pool_event;
use pb::uniswap::types::v1::{Events, Pools};
use std::collections::HashMap;
use substreams::prelude::StoreGetString;
use substreams::store::{
    StoreGet, StoreGetBigDecimal, StoreGetBigInt, StoreGetInt64, StoreMax, StoreMaxInt64, StoreNew,
    StoreSet, StoreSetString,
};
use substreams_ethereum::pb::eth::v2 as eth;

struct PoolAggregator {
    token0_address: String,
    token1_address: String,
    token0_volume: String,
    token1_volume: String,
    swap_count: u32,
    last_sqrt_price: String,
}

impl PoolAggregator {
    fn new(token0: String, token1: String) -> Self {
        Self {
            token0_address: token0,
            token1_address: token1,
            token0_volume: "0".to_string(),
            token1_volume: "0".to_string(),
            swap_count: 0,
            last_sqrt_price: "0".to_string(),
        }
    }

    fn add_swap(&mut self, amount0: &str, amount1: &str, sqrt_price: &str) {
        // Convert to absolute values and add
        let abs_amount0 = amount0.trim_start_matches('-');
        let abs_amount1 = amount1.trim_start_matches('-');

        self.token0_volume = add_strings(&self.token0_volume, abs_amount0);
        self.token1_volume = add_strings(&self.token1_volume, abs_amount1);
        self.swap_count += 1;
        self.last_sqrt_price = sqrt_price.to_string();
    }

    fn sqrt_price_to_price(&self) -> String {
        // Convert sqrt_price to regular price
        // For now, return a simplified calculation
        // In production, use proper BigDecimal math
        if self.last_sqrt_price == "0" {
            return "0".to_string();
        }

        // sqrt_price is Q64.96, so price = (sqrt_price / 2^96)^2
        // This is a simplified version
        let sqrt_val = self.last_sqrt_price.parse::<f64>().unwrap_or(0.0);
        let divisor = (2_f64).powf(96.0);
        let normalized = sqrt_val / divisor;
        let price = normalized * normalized;

        format!("{:.18}", price)
    }
}

// Simple string addition for demo - in production use proper BigDecimal
fn add_strings(a: &str, b: &str) -> String {
    (a.parse::<f64>().unwrap_or(0.0) + b.parse::<f64>().unwrap_or(0.0)).to_string()
}

/// Main map handler that processes Uniswap V3 events and outputs tickers
#[substreams::handlers::map]
fn map_uniswap_ticker_output(
    block: eth::Block,
    pools_created: Pools,
    events: Events,
    _swaps_volume: StoreGetBigDecimal,
    _total_tx_counts: StoreGetBigInt,
    _completed_periods: StoreGetInt64,
    _minute_volumes: StoreGetString,
    rolling_volumes: StoreGetString,
) -> Result<DexOutput, substreams::errors::Error> {
    let mut output = DexOutput {
        dex_info: Some(DexInfo {
            protocol: "uniswap".to_string(),
            version: "v3".to_string(),
            chain: "ethereum".to_string(),
            block_number: block.number,
        }),
        pools_created: vec![],
        tickers: vec![],
    };

    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);

    // Track pool metadata
    let mut pool_metadata: HashMap<String, (String, String)> = HashMap::new();

    // Process pool creation events
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
            factory_address: String::new(), // TODO: Add Uniswap V3 factory address
        };

        // Store metadata
        pool_metadata.insert(
            pool.address.clone(),
            (pool_created.token0.clone(), pool_created.token1.clone()),
        );

        output.pools_created.push(pool_created);
    }

    // Aggregate swaps by pool
    let mut pool_aggregators: HashMap<String, PoolAggregator> = HashMap::new();

    for pool_event in events.pool_events {
        if let Some(pool_event::Type::Swap(swap)) = pool_event.r#type {
            let pool_address = &pool_event.pool_address;

            // Get or create aggregator
            let aggregator = pool_aggregators
                .entry(pool_address.clone())
                .or_insert_with(|| {
                    // Try to get metadata from created pools or use defaults
                    let (token0, token1) =
                        pool_metadata.get(pool_address).cloned().unwrap_or_else(|| {
                            // TODO: In production, fetch from pool store
                            (String::new(), String::new())
                        });
                    PoolAggregator::new(token0, token1)
                });

            aggregator.add_swap(&swap.amount_0, &swap.amount_1, &swap.sqrt_price);
        }
    }

    // Create tickers for pools with activity
    for (pool_address, aggregator) in pool_aggregators {
        // Get 24h rolling volume
        let rolling_key = format!("{pool_address}:rolling");
        let rolling_data = rolling_volumes
            .get_last(&rolling_key)
            .unwrap_or_else(|| "0,0".to_string());
        let rolling_parts: Vec<&str> = rolling_data.split(',').collect();

        // Calculate price before moving values from aggregator
        let close_price = aggregator.sqrt_price_to_price();

        let ticker = PoolTicker {
            pool_address: pool_address.clone(),
            token0_address: aggregator.token0_address,
            token1_address: aggregator.token1_address,
            volume_token0: aggregator.token0_volume,
            volume_token1: aggregator.token1_volume,
            swap_count: aggregator.swap_count,
            close_price,
            volume_24h_token0: rolling_parts.first().unwrap_or(&"0").to_string(),
            volume_24h_token1: rolling_parts.get(1).unwrap_or(&"0").to_string(),
            block_number: block.number,
            timestamp: block_timestamp as u64,
        };

        output.tickers.push(ticker);
    }

    substreams::log::info!(
        "Block {}: {} pools created, {} tickers",
        block.number,
        output.pools_created.len(),
        output.tickers.len()
    );

    Ok(output)
}

// The original VolumeAccumulator for store handlers
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
