// ─────────────────────────────────────────────────────────────────────────────
// Constants and configuration for Uniswap V3 substreams
// ─────────────────────────────────────────────────────────────────────────────

/// Uniswap V3 factory contract address on Ethereum mainnet
pub const FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

/// Duration of each time bucket in seconds (5 minutes)
pub const BUCKET_DURATION_SECONDS: u64 = 300;

/// Number of buckets in a 24-hour period (24h / 5min = 288)
pub const BUCKETS_PER_DAY: u64 = 288;
