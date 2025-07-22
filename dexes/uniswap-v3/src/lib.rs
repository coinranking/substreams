// ─────────────────────────────────────────────────────────────────────────────
// Uniswap V3 — ultra‑fast 24 h rolling volume (cycle‑free, O(active pools))
// ─────────────────────────────────────────────────────────────────────────────
mod pb;

use pb::dex::common::v1::{DexInfo, DexOutput, PoolCreated, PoolTicker};
use pb::uniswap::types::v1::events::pool_event;
use pb::uniswap::types::v1::{Events, Pools};
use std::{collections::HashMap, str::FromStr};
use substreams::{
    scalar::BigDecimal,
    store::{StoreAdd, StoreAddBigDecimal, StoreGet, StoreGetBigDecimal, StoreNew},
};
use substreams_ethereum::pb::eth::v2 as eth;

// On‑chain constants
const FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
const BUCKET_DURATION_SECONDS: u64 = 300; // 5‑minute window
const BUCKETS_PER_DAY: u64 = 288; // 24 h / 5 min

// ───── helpers ───────────────────────────────────────────────────────────────
#[inline]
fn is_zero(big_decimal: &BigDecimal) -> bool {
    big_decimal == &BigDecimal::zero()
}

#[inline]
fn format_bigdecimal(big_decimal: &BigDecimal) -> String {
    // trim to ≤18 decimals, strip trailing zeros
    let mut decimal_string = big_decimal.to_string();
    if let Some(decimal_point_index) = decimal_string.find('.') {
        let truncate_position = usize::min(decimal_point_index + 1 + 18, decimal_string.len());
        decimal_string.truncate(truncate_position);
        while decimal_string.ends_with('0') {
            decimal_string.pop();
        }
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

// ─────────────────────────────────────────────────────────────────────────────
// 1️⃣  Five‑minute bucket accumulator  (add‑only)
// ─────────────────────────────────────────────────────────────────────────────
#[substreams::handlers::store]
fn store_period_volumes(block: eth::Block, events: Events, store: StoreAddBigDecimal) {
    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;
    let period = timestamp_seconds / BUCKET_DURATION_SECONDS;

    for event in events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = event.r#type {
            let pool_address = &event.pool_address;
            let token0_bucket_key = format!("{pool_address}:{period}:t0");
            let token1_bucket_key = format!("{pool_address}:{period}:t1");

            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    store.add(0, &token0_bucket_key, volume);
                }
            }
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    store.add(0, &token1_bucket_key, volume);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2️⃣  Rolling totals (“+Δ − old bucket”)  — O(1) per pool, no cycles
// ─────────────────────────────────────────────────────────────────────────────
#[substreams::handlers::store]
fn store_rolling_deltas(
    block: eth::Block,
    events: Events,
    period_volumes_store: StoreGetBigDecimal,
    rolling_volumes_store: StoreAddBigDecimal, // signed deltas
) {
    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;
    let period = timestamp_seconds / BUCKET_DURATION_SECONDS;
    let evict_period = period.saturating_sub(BUCKETS_PER_DAY);

    // ❶ accumulate positive deltas for this block
    let mut pool_volume_deltas: HashMap<String, (BigDecimal, BigDecimal)> = HashMap::new();
    for event in &events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = &event.r#type {
            let (delta_token0, delta_token1) = pool_volume_deltas
                .entry(event.pool_address.clone())
                .or_insert((BigDecimal::zero(), BigDecimal::zero()));
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    *delta_token0 = delta_token0.clone() + volume;
                }
            }
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    *delta_token1 = delta_token1.clone() + volume;
                }
            }
        }
    }

    // ❷ for each active pool:  −old bucket  +Δblock
    for (pool_address, (delta_token0, delta_token1)) in pool_volume_deltas {
        // subtract exactly ONE bucket (the one leaving the window)
        let token0_evict_key = format!("{pool_address}:{evict_period}:t0");
        let token1_evict_key = format!("{pool_address}:{evict_period}:t1");
        let evicted_volume_token0 = period_volumes_store
            .get_last(&token0_evict_key)
            .unwrap_or_default();
        let evicted_volume_token1 = period_volumes_store
            .get_last(&token1_evict_key)
            .unwrap_or_default();

        if !is_zero(&evicted_volume_token0) {
            rolling_volumes_store.add(
                0,
                &format!("{pool_address}:t0"),
                BigDecimal::zero() - evicted_volume_token0,
            );
        }
        if !is_zero(&evicted_volume_token1) {
            rolling_volumes_store.add(
                0,
                &format!("{pool_address}:t1"),
                BigDecimal::zero() - evicted_volume_token1,
            );
        }

        // now add today's delta
        if !is_zero(&delta_token0) {
            rolling_volumes_store.add(0, &format!("{pool_address}:t0"), delta_token0);
        }
        if !is_zero(&delta_token1) {
            rolling_volumes_store.add(0, &format!("{pool_address}:t1"), delta_token1);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3️⃣  Final ticker map (reads rolling totals)
// ─────────────────────────────────────────────────────────────────────────────
#[substreams::handlers::map]
fn map_uniswap_ticker_output(
    block: eth::Block,
    pools: Pools,
    events: Events,
    rolling_volumes_store: StoreGetBigDecimal,
) -> Result<DexOutput, substreams::errors::Error> {
    let mut dex_output = DexOutput {
        dex_info: Some(DexInfo {
            protocol: "uniswap".into(),
            version: "v3".into(),
            chain: "ethereum".into(),
            block_number: block.number,
            factory_address: FACTORY.into(),
        }),
        pools_created: vec![],
        tickers: vec![],
    };

    // PoolCreated pass‑through
    for pool in pools.pools {
        dex_output.pools_created.push(PoolCreated {
            pool_address: pool.address.clone(),
            token0: pool
                .token0
                .as_ref()
                .map(|token| token.address.clone())
                .unwrap_or_default(),
            token1: pool
                .token1
                .as_ref()
                .map(|token| token.address.clone())
                .unwrap_or_default(),
            fee: pool.fee_tier.parse::<u32>().unwrap_or_default(),
            block_number: block.number,
            transaction_hash: pool.transaction_id.clone(),
        });
    }

    // per‑block aggregation (cheap)
    let mut pool_aggregations: HashMap<String, (BigDecimal, BigDecimal, u32)> = HashMap::new();
    for event in events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = event.r#type {
            let entry = pool_aggregations
                .entry(event.pool_address.clone())
                .or_insert((BigDecimal::zero(), BigDecimal::zero(), 0));
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                entry.0 = entry.0.clone() + volume;
            }
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                entry.1 = entry.1.clone() + volume;
            }
            entry.2 += 1;
        }
    }

    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;

    for (pool_address, (current_volume_token0, current_volume_token1, swaps)) in pool_aggregations {
        let rolling_volume_token0 = rolling_volumes_store
            .get_last(&format!("{pool_address}:t0"))
            .unwrap_or_default();
        let rolling_volume_token1 = rolling_volumes_store
            .get_last(&format!("{pool_address}:t1"))
            .unwrap_or_default();

        dex_output.tickers.push(PoolTicker {
            pool_address,
            volume_token0: format_bigdecimal(&current_volume_token0),
            volume_token1: format_bigdecimal(&current_volume_token1),
            swap_count: swaps,
            close_price: "0".into(), // √P kept raw for speed
            volume_24h_token0: format_bigdecimal(&rolling_volume_token0),
            volume_24h_token1: format_bigdecimal(&rolling_volume_token1),
            block_number: block.number,
            timestamp: timestamp_seconds,
        });
    }

    Ok(dex_output)
}
