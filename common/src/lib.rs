use num_bigint::BigInt as NumBigInt;
use substreams::scalar::BigInt;

/// Format BigInt as a string, handling potential edge cases
#[inline]
pub fn format_bigint(value: &BigInt) -> String {
    value.to_string()
}

/// Ensure address has 0x prefix
#[inline]
pub fn ensure_0x_prefix(address: &str) -> String {
    if address.starts_with("0x") || address.starts_with("0X") {
        address.to_string()
    } else {
        format!("0x{address}")
    }
}

/// Convert signed int256 bytes to BigInt
///
/// ## EVM Integer Storage:
/// - Integers in the EVM are stored as 32-byte (256-bit) words
/// - Signed integers use two's complement representation
/// - This function handles the conversion from raw bytes to BigInt
///
/// ## Parameters:
/// - `bytes`: Must be exactly 32 bytes representing a signed 256-bit integer
///
/// ## Returns:
/// - The BigInt representation of the integer, or 0 if invalid input
#[inline]
pub fn int256_to_bigint(bytes: &[u8]) -> BigInt {
    if bytes.len() != 32 {
        return BigInt::zero();
    }

    let bigint = NumBigInt::from_signed_bytes_be(bytes);
    BigInt::from(bigint)
}

/// Convert unsigned uint160 bytes (stored in 32 bytes) to BigInt
///
/// ## Why uint160:
/// - Ethereum addresses are 160 bits (20 bytes)
/// - sqrtPriceX96 in Uniswap V3 is stored as uint160
/// - When stored in a 32-byte word, uint160 is right-aligned (last 20 bytes)
///
/// ## Parameters:
/// - `bytes`: Must be exactly 32 bytes with uint160 in the last 20 bytes
///
/// ## Returns:
/// - The BigInt representation of the uint160 value, or 0 if invalid input
#[inline]
pub fn uint160_to_bigint(bytes: &[u8]) -> BigInt {
    if bytes.len() != 32 {
        return BigInt::zero();
    }

    // uint160 is stored in the last 20 bytes of the 32-byte word
    let start = bytes.len().saturating_sub(20);
    let bigint = NumBigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[start..]);
    BigInt::from(bigint)
}

/// Convert unsigned uint256 bytes to BigInt
///
/// ## Parameters:
/// - `bytes`: Must be exactly 32 bytes representing an unsigned 256-bit integer
///
/// ## Returns:
/// - The BigInt representation of the uint256 value, or 0 if invalid input
#[inline]
pub fn uint256_to_bigint(bytes: &[u8]) -> BigInt {
    if bytes.len() != 32 {
        return BigInt::zero();
    }

    let bigint = NumBigInt::from_bytes_be(num_bigint::Sign::Plus, bytes);
    BigInt::from(bigint)
}

/// Convert unsigned uint112 bytes (stored in 32 bytes) to BigInt
///
/// ## Why uint112:
/// - Uniswap V2 uses uint112 for reserve values
/// - When stored in a 32-byte word, uint112 is right-aligned (last 14 bytes)
///
/// ## Parameters:
/// - `bytes`: Must be exactly 32 bytes with uint112 in the last 14 bytes
///
/// ## Returns:
/// - The BigInt representation of the uint112 value, or 0 if invalid input
#[inline]
pub fn uint112_to_bigint(bytes: &[u8]) -> BigInt {
    if bytes.len() != 32 {
        return BigInt::zero();
    }

    // uint112 is stored in the last 14 bytes of the 32-byte word
    let start = bytes.len().saturating_sub(14);
    let bigint = NumBigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[start..]);
    BigInt::from(bigint)
}

/// Calculate sqrtPriceX96 from reserve amounts
///
/// ## Uniswap V2 to V3 Price Conversion:
/// - Converts V2-style reserves to V3-style sqrtPriceX96 format
/// - Formula: sqrtPriceX96 = sqrt(reserve1 / reserve0) * 2^96
/// - Implemented as: sqrt(reserve1 * 2^192 / reserve0) to avoid precision loss
///
/// ## Parameters:
/// - `reserve0`: Reserve amount of token0 (raw units)
/// - `reserve1`: Reserve amount of token1 (raw units)
///
/// ## Returns:
/// - The sqrtPriceX96 value representing price of token1 in terms of token0
/// - Returns 0 if reserve0 is zero (to avoid division by zero)
pub fn calculate_sqrt_price_x96(reserve0: &BigInt, reserve1: &BigInt) -> BigInt {
    // Avoid division by zero
    if reserve0.clone() == BigInt::zero() {
        return BigInt::zero();
    }

    // Convert to num_bigint for sqrt calculation
    let r0 = NumBigInt::try_from(reserve0.clone()).unwrap_or_default();
    let r1 = NumBigInt::try_from(reserve1.clone()).unwrap_or_default();

    // Calculate: sqrt(reserve1 * 2^192 / reserve0)
    // Shift left by 192 bits is equivalent to multiplying by 2^192
    let numerator = r1 << 192;
    let ratio: NumBigInt = numerator / r0;
    let sqrt_ratio = ratio.sqrt();

    // Convert back to substreams BigInt
    BigInt::from(sqrt_ratio)
}
