// ─────────────────────────────────────────────────────────────────────────────
// Rolling 24h volume calculator with O(1) performance
// ─────────────────────────────────────────────────────────────────────────────

use crate::constants::{BUCKETS_PER_DAY, BUCKET_DURATION_SECONDS};
use crate::pb::uniswap::types::v1::events::pool_event;
use crate::pb::uniswap::types::v1::Events;
use crate::utils::is_zero;
use std::collections::HashMap;
use std::str::FromStr;
use substreams::scalar::BigDecimal;
use substreams::store::{StoreAdd, StoreAddBigDecimal, StoreGet, StoreGetBigDecimal, StoreNew};
use substreams_ethereum::pb::eth::v2 as eth;

/// Store handler that maintains 24h rolling volume totals
/// Uses an O(1) algorithm by adding current swap volumes and subtracting
/// volumes that are exactly 24 hours old (288 periods)
#[substreams::handlers::store]
pub fn store_rolling_deltas(
    block: eth::Block,
    events: Events,
    period_volumes_store: StoreGetBigDecimal,
    rolling_volumes_store: StoreAddBigDecimal,
) {
    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;

    let period = timestamp_seconds / BUCKET_DURATION_SECONDS;
    let evict_period = period.saturating_sub(BUCKETS_PER_DAY);

    // Accumulate positive deltas for this block
    let mut pool_volume_deltas: HashMap<String, (BigDecimal, BigDecimal)> = HashMap::new();

    for event in &events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = &event.r#type {
            let (delta_token0, delta_token1) = pool_volume_deltas
                .entry(event.pool_address.clone())
                .or_insert((BigDecimal::zero(), BigDecimal::zero()));

            // Accumulate token0 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    *delta_token0 = delta_token0.clone() + volume;
                }
            }

            // Accumulate token1 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    *delta_token1 = delta_token1.clone() + volume;
                }
            }
        }
    }

    // For each active pool: subtract old bucket and add new delta
    for (pool_address, (delta_token0, delta_token1)) in pool_volume_deltas {
        // Subtract volumes from exactly 24 hours ago
        let token0_evict_key = format!("{pool_address}:{evict_period}:t0");
        let token1_evict_key = format!("{pool_address}:{evict_period}:t1");

        let evicted_volume_token0 = period_volumes_store
            .get_last(&token0_evict_key)
            .unwrap_or_default();
        let evicted_volume_token1 = period_volumes_store
            .get_last(&token1_evict_key)
            .unwrap_or_default();

        // Subtract evicted volumes
        if !is_zero(&evicted_volume_token0) {
            rolling_volumes_store.add(
                0,
                format!("{pool_address}:t0"),
                BigDecimal::zero() - evicted_volume_token0,
            );
        }
        if !is_zero(&evicted_volume_token1) {
            rolling_volumes_store.add(
                0,
                format!("{pool_address}:t1"),
                BigDecimal::zero() - evicted_volume_token1,
            );
        }

        // Add current period's delta
        if !is_zero(&delta_token0) {
            rolling_volumes_store.add(0, format!("{pool_address}:t0"), delta_token0);
        }
        if !is_zero(&delta_token1) {
            rolling_volumes_store.add(0, format!("{pool_address}:t1"), delta_token1);
        }
    }
}
