use crate::pb::evm::common::v1::SupplyDelta;
use dex_common::{ensure_0x_prefix, format_bigint, uint256_to_bigint};
use substreams::Hex;
use substreams_ethereum::block_view::LogView;

const ZERO_ADDRESS: [u8; 32] = [0u8; 32];

/// Process Transfer events, returning SupplyDelta only for mints/burns
pub fn process_transfer_event(
    log: &LogView,
    block_number: u64,
    timestamp: u64,
) -> Option<SupplyDelta> {
    // Transfer event has 3 topics: sig, from (indexed), to (indexed)
    // and data contains: value (uint256)
    if log.topics().len() < 3 || log.data().len() < 32 {
        return None;
    }

    let from = log.topics()[1].as_slice();
    let to = log.topics()[2].as_slice();

    // Check for mint (from = 0x0) or burn (to = 0x0)
    let is_mint = from == ZERO_ADDRESS;
    let is_burn = to == ZERO_ADDRESS;

    if !is_mint && !is_burn {
        return None; // Regular transfer, skip
    }

    let amount = uint256_to_bigint(&log.data()[0..32]);
    let token_address = ensure_0x_prefix(&Hex(&log.log.address).to_string());

    Some(SupplyDelta {
        token_address,
        amount: format_bigint(&amount),
        is_mint,
        block_number,
        timestamp,
    })
}
