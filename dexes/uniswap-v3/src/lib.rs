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

// Uniswap V3 constants
const UNISWAP_V3_FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

// Time period configuration for volume tracking
const BUCKET_DURATION_SECONDS: u64 = 300; // 5 minutes per bucket
const BUCKETS_PER_DAY: u64 = 288; // 24 hours / 5 minutes = 288 buckets

struct PoolAggregator {
    token0_volume: String,
    token1_volume: String,
    swap_count: u32,
    last_sqrt_price: String,
}

impl PoolAggregator {
    fn new() -> Self {
        Self {
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
    _period_volumes: StoreGetString,
    rolling_volumes: StoreGetString,
) -> Result<DexOutput, substreams::errors::Error> {
    let mut output = DexOutput {
        dex_info: Some(DexInfo {
            protocol: "uniswap".to_string(),
            version: "v3".to_string(),
            chain: "ethereum".to_string(),
            block_number: block.number,
            factory_address: UNISWAP_V3_FACTORY.to_string(),
        }),
        pools_created: vec![],
        tickers: vec![],
    };

    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);

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
        };

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
                .or_insert_with(PoolAggregator::new);

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
    let current_period = (block_timestamp as u64) / BUCKET_DURATION_SECONDS;

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
        output.max(0, &pool_address, current_period as i64);
    }
}

/// Store handler that accumulates swap volumes into period buckets.
/// Each pool has BUCKETS_PER_DAY buckets for a 24-hour circular buffer.
#[substreams::handlers::store]
fn store_period_volumes(
    block: eth::Block,
    events: Events,
    period_volumes: StoreGetString,
    store: StoreSetString,
) {
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);
    let current_period = (block_timestamp as u64) / BUCKET_DURATION_SECONDS;

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

    // Process volumes for each pool
    for (pool_address, accumulator) in pool_volumes {
        let bucket_idx = current_period % BUCKETS_PER_DAY;
        let bucket_key = format!("{pool_address}:bucket:{bucket_idx}");
        let period_key = format!("{pool_address}:bucket:{bucket_idx}:period");

        // Check if this bucket belongs to an old period (needs reset)
        let existing_period = period_volumes.get_last(&period_key);
        let needs_reset = if let Some(period_str) = existing_period {
            if let Ok(last_period) = period_str.parse::<u64>() {
                current_period - last_period >= BUCKETS_PER_DAY
            } else {
                false
            }
        } else {
            false
        };

        // Get existing volumes or start fresh
        let (mut total_token0, mut total_token1) = if needs_reset {
            substreams::log::debug!(
                "Reset bucket {} for pool {} (new period {})",
                bucket_idx,
                pool_address,
                current_period
            );
            ("0".to_string(), "0".to_string())
        } else {
            // Get existing volumes from same period
            let existing = period_volumes
                .get_last(&bucket_key)
                .unwrap_or_else(|| "0,0".to_string());
            let parts: Vec<&str> = existing.split(',').collect();
            (
                parts.first().unwrap_or(&"0").to_string(),
                parts.get(1).unwrap_or(&"0").to_string(),
            )
        };

        // Add new volumes
        total_token0 = add_strings(&total_token0, &accumulator.token0_volume);
        total_token1 = add_strings(&total_token1, &accumulator.token1_volume);

        // Store updated volumes
        let bucket_value = format!("{},{}", total_token0, total_token1);
        store.set(0, &bucket_key, &bucket_value);

        // Update period metadata
        store.set(0, &period_key, &current_period.to_string());

        substreams::log::debug!(
            "Updated volume for pool {} in bucket {} (period {}): {}",
            pool_address,
            bucket_idx,
            current_period,
            bucket_value
        );
    }
}

/// Store handler that maintains 24-hour rolling volumes for each pool.
/// Updates by summing all bucket values within the 24-hour window.
#[substreams::handlers::store]
fn store_rolling_volumes(
    block: eth::Block,
    events: Events,
    _completed_periods: StoreGetInt64,
    period_volumes: StoreGetString,
    store: StoreSetString,
) {
    let block_timestamp = block
        .header
        .as_ref()
        .map(|h| h.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0))
        .unwrap_or(0);
    let current_period = (block_timestamp as u64) / BUCKET_DURATION_SECONDS;

    // Track pools with activity to update their rolling volumes
    let mut active_pools = std::collections::HashSet::new();

    for pool_event in &events.pool_events {
        if let Some(pool_event::Type::Swap(_)) = &pool_event.r#type {
            active_pools.insert(pool_event.pool_address.clone());
        }
    }

    // Update rolling volumes for active pools
    for pool_address in active_pools {
        // Always recalculate rolling volume from all buckets
        let rolling_key = format!("{pool_address}:rolling");

        // Sum all valid buckets for 24-hour volume
        let mut total_token0 = "0".to_string();
        let mut total_token1 = "0".to_string();

        // Check each bucket to see if it's within the last 24 hours
        for bucket_idx in 0..BUCKETS_PER_DAY {
            let bucket_key = format!("{pool_address}:bucket:{bucket_idx}");
            let period_key = format!("{pool_address}:bucket:{bucket_idx}:period");

            // Get the period when this bucket was last updated
            if let Some(period_str) = period_volumes.get_last(&period_key) {
                if let Ok(bucket_period) = period_str.parse::<u64>() {
                    // Only include if within last 24 hours
                    if current_period >= bucket_period
                        && current_period.saturating_sub(bucket_period) < BUCKETS_PER_DAY
                    {
                        // Get volumes from the bucket
                        let bucket_value = period_volumes
                            .get_last(&bucket_key)
                            .unwrap_or_else(|| "0,0".to_string());
                        let parts: Vec<&str> = bucket_value.split(',').collect();

                        total_token0 = add_strings(&total_token0, parts.first().unwrap_or(&"0"));
                        total_token1 = add_strings(&total_token1, parts.get(1).unwrap_or(&"0"));
                    }
                }
            }
        }

        let new_rolling_value = format!("{},{}", total_token0, total_token1);

        substreams::log::debug!(
            "Updating rolling volume for pool {}: {}",
            pool_address,
            new_rolling_value
        );
        store.set(0, &rolling_key, &new_rolling_value);
    }
}
