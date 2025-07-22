// ─────────────────────────────────────────────────────────────────────────────
// Utility functions for Uniswap V3 substreams
// ─────────────────────────────────────────────────────────────────────────────

use substreams::scalar::BigDecimal;

/// Check if a BigDecimal value is zero
#[inline]
pub fn is_zero(big_decimal: &BigDecimal) -> bool {
    big_decimal == &BigDecimal::zero()
}

/// Format a BigDecimal to a string with at most 18 decimal places
/// Removes trailing zeros and decimal point if unnecessary
#[inline]
pub fn format_bigdecimal(big_decimal: &BigDecimal) -> String {
    let mut decimal_string = big_decimal.to_string();

    if let Some(decimal_point_index) = decimal_string.find('.') {
        // Truncate to maximum 18 decimal places
        let truncate_position = usize::min(decimal_point_index + 1 + 18, decimal_string.len());
        decimal_string.truncate(truncate_position);

        // Remove trailing zeros
        while decimal_string.ends_with('0') {
            decimal_string.pop();
        }

        // Remove decimal point if no decimals remain
        if decimal_string.ends_with('.') {
            decimal_string.pop();
        }
    }

    if decimal_string.is_empty() {
        "0".into()
    } else {
        decimal_string
    }
}
