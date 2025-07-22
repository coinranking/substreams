// ─────────────────────────────────────────────────────────────────────────────
// Five-minute bucket volume accumulator
// ─────────────────────────────────────────────────────────────────────────────

use crate::constants::BUCKET_DURATION_SECONDS;
use crate::pb::uniswap::types::v1::events::pool_event;
use crate::pb::uniswap::types::v1::Events;
use crate::utils::is_zero;
use std::str::FromStr;
use substreams::scalar::BigDecimal;
use substreams::store::{StoreAdd, StoreAddBigDecimal, StoreNew};
use substreams_ethereum::pb::eth::v2 as eth;

/// Store handler that accumulates swap volumes into 5-minute buckets
/// Each bucket stores the total volume for each token in each pool
#[substreams::handlers::store]
pub fn store_period_volumes(block: eth::Block, events: Events, store: StoreAddBigDecimal) {
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

            // Parse and accumulate token0 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    store.add(0, &token0_bucket_key, volume);
                }
            }

            // Parse and accumulate token1 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                if !is_zero(&volume) {
                    store.add(0, &token1_bucket_key, volume);
                }
            }
        }
    }
}
