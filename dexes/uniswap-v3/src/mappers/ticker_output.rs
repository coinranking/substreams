// ─────────────────────────────────────────────────────────────────────────────
// Final ticker output mapper
// ─────────────────────────────────────────────────────────────────────────────

use crate::constants::FACTORY;
use crate::pb::dex::common::v1::{DexInfo, DexOutput, PoolCreated, PoolTicker};
use crate::pb::uniswap::types::v1::events::pool_event;
use crate::pb::uniswap::types::v1::{Events, Pools};
use crate::utils::format_bigdecimal;
use std::collections::HashMap;
use std::str::FromStr;
use substreams::scalar::BigDecimal;
use substreams::store::{StoreGet, StoreGetBigDecimal};
use substreams_ethereum::pb::eth::v2 as eth;

/// Map handler that generates the final DexOutput with ticker information
/// Combines current block swap data with 24h rolling volumes
#[substreams::handlers::map]
pub fn map_uniswap_ticker_output(
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

    // Pass through pool creation events
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

    // Aggregate current block's swap volumes
    let mut pool_aggregations: HashMap<String, (BigDecimal, BigDecimal, u32)> = HashMap::new();

    for event in events.pool_events {
        if let Some(pool_event::Type::Swap(swap_event)) = event.r#type {
            let entry = pool_aggregations
                .entry(event.pool_address.clone())
                .or_insert((BigDecimal::zero(), BigDecimal::zero(), 0));

            // Accumulate token0 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_0.trim_start_matches('-')) {
                entry.0 = entry.0.clone() + volume;
            }

            // Accumulate token1 volume
            if let Ok(volume) = BigDecimal::from_str(swap_event.amount_1.trim_start_matches('-')) {
                entry.1 = entry.1.clone() + volume;
            }

            entry.2 += 1; // Increment swap count
        }
    }

    let timestamp_seconds = block
        .header
        .as_ref()
        .and_then(|header| header.timestamp.as_ref())
        .map(|timestamp| timestamp.seconds)
        .unwrap_or(0) as u64;

    // Generate ticker data for each pool that had swaps
    for (pool_address, (current_volume_token0, current_volume_token1, swaps)) in pool_aggregations {
        // Fetch 24h rolling volumes from store
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
            close_price: "0".into(), // Price calculation left for downstream processing
            volume_24h_token0: format_bigdecimal(&rolling_volume_token0),
            volume_24h_token1: format_bigdecimal(&rolling_volume_token1),
            block_number: block.number,
            timestamp: timestamp_seconds,
        });
    }

    Ok(dex_output)
}
