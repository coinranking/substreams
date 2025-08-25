use num_bigint::BigInt as NumBigInt;
use std::str::FromStr;
use substreams::scalar::{BigDecimal, BigInt};

/// Calculate actual price from sqrtPriceX96 format used by Uniswap V3 and compatible DEXes
///
/// ## Why DEXes store price as sqrtPriceX96:
///
/// 1. **Gas Efficiency**: Calculations with square roots reduce computational complexity
/// 2. **Solidity Limitations**: No floating-point numbers in Solidity, so fixed-point is used
/// 3. **Q64.96 Format**: A 256-bit number where 64 bits represent the integer part and 96 bits
///    represent the fractional part, allowing for high precision without decimals
/// 4. **Storage Optimization**: 2^96 was chosen as the largest precision that fits efficiently
///    in contract storage slots, allowing multiple values to be packed together
///
/// ## The Math:
/// - sqrtPriceX96 = sqrt(price) * 2^96
/// - Therefore: price = (sqrtPriceX96 / 2^96)^2
/// - This gives us the price of token1 in terms of token0
///
/// ## Example:
/// If sqrtPriceX96 = 79228162514264337593543950336 (which is 2^96)
/// Then price = (2^96 / 2^96)^2 = 1^2 = 1 (tokens are equal value)
#[inline]
pub fn calculate_price_from_sqrt_x96(sqrt_price_x96: &BigDecimal) -> BigDecimal {
    // 2^96 as a decimal constant
    let two_96 = BigDecimal::from_str("79228162514264337593543950336").unwrap();

    // Divide by 2^96 to get the actual square root of price
    let sqrt_price = sqrt_price_x96.clone() / two_96;

    // Square it to get the actual price
    sqrt_price.clone() * sqrt_price
}

/// Format BigDecimal with at most 18 decimal places, removing trailing zeros
/// Handles scientific notation gracefully
#[inline]
pub fn format_bigdecimal(value: &BigDecimal) -> String {
    let mut s = value.to_string();

    // Handle scientific notation - return as-is
    if s.contains('e') || s.contains('E') {
        return s;
    }

    if let Some(decimal_point_index) = s.find('.') {
        // Truncate to maximum 18 decimal places
        let truncate_position = usize::min(decimal_point_index + 1 + 18, s.len());
        s.truncate(truncate_position);

        // Remove trailing zeros
        while s.ends_with('0') {
            s.pop();
        }

        // Remove decimal point if no decimals remain
        if s.ends_with('.') {
            s.pop();
        }
    }

    if s.is_empty() {
        "0".to_string()
    } else {
        s
    }
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

/// Convert signed int256 bytes to BigDecimal
///
/// ## EVM Integer Storage:
/// - Integers in the EVM are stored as 32-byte (256-bit) words
/// - Signed integers use two's complement representation
/// - This function handles the conversion from raw bytes to a decimal value
///
/// ## Parameters:
/// - `bytes`: Must be exactly 32 bytes representing a signed 256-bit integer
///
/// ## Returns:
/// - The decimal representation of the integer, or 0 if invalid input
#[inline]
pub fn int256_to_bigdecimal(bytes: &[u8]) -> BigDecimal {
    if bytes.len() != 32 {
        return BigDecimal::zero();
    }

    let bigint = NumBigInt::from_signed_bytes_be(bytes);
    BigDecimal::from(BigInt::from(bigint))
}

/// Convert unsigned uint160 bytes (stored in 32 bytes) to BigDecimal
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
/// - The decimal representation of the uint160 value, or 0 if invalid input
#[inline]
pub fn uint160_to_bigdecimal(bytes: &[u8]) -> BigDecimal {
    if bytes.len() != 32 {
        return BigDecimal::zero();
    }

    // uint160 is stored in the last 20 bytes of the 32-byte word
    let start = bytes.len().saturating_sub(20);
    let bigint = NumBigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[start..]);
    BigDecimal::from(BigInt::from(bigint))
}
