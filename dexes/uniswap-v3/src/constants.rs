// ─────────────────────────────────────────────────────────────────────────────
// Constants and configuration for Uniswap V3 substreams
// ─────────────────────────────────────────────────────────────────────────────

/// Duration of each time bucket in seconds (5 minutes)
pub const BUCKET_DURATION_SECONDS: u64 = 300;

/// Number of buckets in a 24-hour period (24h / 5min = 288)
pub const BUCKETS_PER_DAY: u64 = 288;
