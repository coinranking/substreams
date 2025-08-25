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