use substreams::scalar::BigInt;

/// Aggregation struct for pool data across all DEX versions
#[derive(Clone)]
pub struct SwapAggregation {
    pub volume_token0: BigInt,
    pub volume_token1: BigInt,
    pub swap_count: u32,
    pub last_sqrt_price: BigInt,
}

impl Default for SwapAggregation {
    fn default() -> Self {
        Self {
            volume_token0: BigInt::zero(),
            volume_token1: BigInt::zero(),
            swap_count: 0,
            last_sqrt_price: BigInt::zero(),
        }
    }
}
