use super::hypercore_api::{Fill, MetaResponse, SpotMetaResponse};
use tracing::debug;

/// Validates if a size can be properly truncated to the required sz_decimals
/// Returns true if the size is valid (truncated size > 0), false otherwise
///
/// # Arguments
/// * `size` - The size to validate
/// * `sz_decimals` - The size decimals required by the asset
///
/// # Examples
/// ```
/// use oe_client_hyperliquid::validate_size;
/// // sz_decimals = 0 (integer only)
/// assert!(validate_size(1.0, 0)); // OK: truncates to 1
/// assert!(validate_size(1.9, 0)); // OK: truncates to 1 (no % check needed)
/// assert!(!validate_size(0.9, 0)); // FAIL: truncates to 0
///
/// // sz_decimals = 2
/// assert!(validate_size(1.234, 2)); // OK: truncates to 1.23
/// assert!(!validate_size(0.004, 2)); // FAIL: truncates to 0
/// ```
pub fn validate_size(size: f64, sz_decimals: u8) -> bool {
	// Truncate (round down) to sz_decimals
	let multiplier = 10f64.powi(sz_decimals as i32);
	let truncated = (size * multiplier).floor() / multiplier;

	// Only check if truncated size is greater than zero
	truncated > 0.0
}

/// Validates if a size meets minimum requirements for trading
/// Checks if the size truncates to a valid non-zero value
///
/// # Arguments
/// * `size` - The size to validate
/// * `sz_decimals` - The size decimals required by the asset
/// * `min_size` - Optional minimum size (default: smallest unit based on sz_decimals)
///
/// # Returns
/// Ok(()) if valid, Err(String) with reason if invalid
pub fn validate_trade_size(
	size: f64,
	sz_decimals: u8,
	min_size: Option<f64>,
) -> Result<(), String> {
	// Determine minimum size if not provided
	let min_size = min_size.unwrap_or_else(|| {
		// Minimum is 1 unit at the given decimal precision
		10f64.powi(-(sz_decimals as i32))
	});

	if size < min_size {
		return Err(format!(
			"Size {} is below minimum trade size {} (sz_decimals={})",
			size, min_size, sz_decimals
		));
	}

	if !validate_size(size, sz_decimals) {
		let multiplier = 10f64.powi(sz_decimals as i32);
		let truncated = (size * multiplier).floor() / multiplier;
		return Err(format!(
			"Size {} would truncate to {} with sz_decimals={}, which is invalid (must be > 0)",
			size, truncated, sz_decimals
		));
	}

	Ok(())
}

/// Get the asset ID for a spot trading pair (ticker/USDC)
///
/// # Arguments
/// * `ticker` - The token ticker (e.g., "BTC", "ETH")
/// * `spot_meta` - The spot market metadata from the API
///
/// # Returns
/// The asset ID for the spot pair, or an error if not found
pub fn get_spot_asset_id(ticker: &str, spot_meta: &SpotMetaResponse) -> Result<u32, String> {
	let token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(ticker))
		.ok_or_else(|| format!("Token {} not found in spot meta", ticker))?;

	let usdc_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case("USDC"))
		.ok_or_else(|| "USDC token not found in spot meta".to_string())?;

	let pair = spot_meta
		.universe
		.iter()
		.find(|p| p.tokens.contains(&token.index) && p.tokens.contains(&usdc_token.index))
		.ok_or_else(|| format!("No spot pair found for {}/USDC", ticker))?;

	Ok(10000 + pair.index)
}

/// Get the asset ID for a perpetual futures contract
///
/// # Arguments
/// * `ticker` - The asset ticker (e.g., "BTC", "ETH")
/// * `meta` - The perp market metadata from the API
///
/// # Returns
/// The asset ID for the perp, or an error if not found
pub fn get_perp_asset_id(ticker: &str, meta: &MetaResponse) -> Result<u32, String> {
	let asset = meta
		.universe
		.iter()
		.enumerate()
		.find(|(_, a)| a.name.eq_ignore_ascii_case(ticker))
		.ok_or_else(|| format!("Perp asset {} not found in meta", ticker))?;

	Ok(asset.0 as u32)
}

/// Calculate the USDC received from a spot sell fill
/// For a sell order: USDC received = price * size - fee (if fee is in USDC)
/// Returns the net USDC amount received
pub fn usdc_from_spot_fill(fill: &Fill) -> Result<f64, String> {
	// Parse price and size
	let price = fill
		.px
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill price: {}", e))?;
	let size = fill
		.sz
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill size: {}", e))?;
	let fee = fill
		.fee
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill fee: {}", e))?;

	// Verify it's a sell order
	if !fill.side.eq_ignore_ascii_case("A") && !fill.dir.to_lowercase().contains("sell") {
		return Err(format!("Fill is not a sell order: side={}, dir={}", fill.side, fill.dir));
	}

	// Calculate gross USDC from the trade (price * size for sell)
	let gross_usdc = price * size;

	// Subtract fee if it's in USDC
	let net_usdc = if fill.fee_token.eq_ignore_ascii_case("USDC") {
		gross_usdc - fee
	} else {
		// If fee is not in USDC, we don't subtract it from the USDC amount
		// but we log a warning
		debug!("Warning: Fee is in {} not USDC, returning gross USDC amount", fill.fee_token);
		gross_usdc
	};

	debug!(
		"Spot sell fill: price={}, size={}, fee={} {}, gross_usdc={}, net_usdc={}",
		price, size, fee, fill.fee_token, gross_usdc, net_usdc
	);

	Ok(net_usdc)
}

/// Clamps a price to comply with HyperLiquid tick size rules.
///
/// Price rules:
/// - Maximum 5 significant figures (excluding leading zeros)
/// - Maximum decimal places = MAX_DECIMALS - sz_decimals
///   - For perps: MAX_DECIMALS = 6
///   - For spot: MAX_DECIMALS = 8
/// - Integer prices are always allowed (regardless of significant figures)
///
/// # Arguments
/// * `price` - The price to clamp
/// * `sz_decimals` - The size decimals of the asset
/// * `is_spot` - Whether this is a spot market (true) or perp market (false)
///
/// # Returns
/// The clamped price as a string
pub fn clamp_price(price: f64, sz_decimals: u8, is_spot: bool) -> String {
	const MAX_SIGNIFICANT_FIGURES: usize = 5;
	const PERP_MAX_DECIMALS: u8 = 6;
	const SPOT_MAX_DECIMALS: u8 = 8;

	// Determine max decimal places based on market type
	let max_decimals = if is_spot { SPOT_MAX_DECIMALS } else { PERP_MAX_DECIMALS };
	let max_price_decimals = max_decimals.saturating_sub(sz_decimals);

	// Check if the price is effectively an integer
	if (price - price.floor()).abs() < 1e-10 {
		// Integer prices are always allowed
		return format!("{:.0}", price);
	}

	// Convert to string to analyze significant figures
	let price_str = format!("{:.15}", price); // Use high precision initially

	// Count significant figures (excluding leading zeros)
	let mut _sig_figs = 0;
	let mut counting = false;
	let mut _decimal_point_seen = false;
	let mut _decimal_places = 0;

	for ch in price_str.chars() {
		if ch == '.' {
			_decimal_point_seen = true;
		} else if ch.is_ascii_digit() {
			if ch != '0' || counting {
				_sig_figs += 1;
				counting = true;
			}
			if _decimal_point_seen {
				_decimal_places += 1;
			}
		}
	}

	// Determine the limiting factor: significant figures or decimal places
	let target_decimals = if price >= 1.0 {
		// For prices >= 1, significant figures typically limit first
		// Calculate how many decimal places we can have with 5 sig figs
		let integer_part = price.floor();
		let integer_digits =
			if integer_part == 0.0 { 0 } else { (integer_part.log10().floor() as usize) + 1 };
		let decimals_from_sig_figs = MAX_SIGNIFICANT_FIGURES.saturating_sub(integer_digits);

		// Take the minimum of sig fig limit and max decimals limit
		std::cmp::min(decimals_from_sig_figs, max_price_decimals as usize)
	} else {
		// For prices < 1, we need to count leading zeros after decimal point
		let mut leading_zeros = 0;
		let mut after_decimal = false;
		for ch in price_str.chars() {
			if ch == '.' {
				after_decimal = true;
			} else if after_decimal {
				if ch == '0' {
					leading_zeros += 1;
				} else {
					break;
				}
			}
		}

		// With 5 sig figs, we can have leading_zeros + 5 total decimal places
		let decimals_from_sig_figs = leading_zeros + MAX_SIGNIFICANT_FIGURES;

		// Take the minimum of sig fig limit and max decimals limit
		std::cmp::min(decimals_from_sig_figs, max_price_decimals as usize)
	};

	// Truncate to target decimal places (drop extra digits, don't round)
	let multiplier = 10f64.powi(target_decimals as i32);
	let truncated = (price * multiplier).floor() / multiplier;

	// Format with the appropriate number of decimals, removing trailing zeros
	let formatted = format!("{:.prec$}", truncated, prec = target_decimals);

	// Remove trailing zeros and decimal point if not needed
	let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

	// If we ended up with an empty string or just "0", return the price as is
	if trimmed.is_empty() || trimmed == "0" {
		format!("{}", price)
	} else {
		trimmed.to_string()
	}
}

/// Clamps a size to comply with HyperLiquid lot size rules.
///
/// Size rules:
/// - Sizes are truncated (rounded down) to the sz_decimals of the asset
/// - Example: if sz_decimals = 3, then 1.001 is valid but 1.0001 truncates to 1.000
///
/// # Arguments
/// * `size` - The size to clamp
/// * `sz_decimals` - The size decimals of the asset
///
/// # Returns
/// The clamped size as a string
pub fn clamp_size(size: f64, sz_decimals: u8) -> String {
	// Truncate (round down) to sz_decimals
	let multiplier = 10f64.powi(sz_decimals as i32);
	let truncated = (size * multiplier).floor() / multiplier;

	// Format with the appropriate number of decimals
	if sz_decimals == 0 {
		// For integer sizes, format as integer
		format!("{:.0}", truncated)
	} else {
		let formatted = format!("{:.prec$}", truncated, prec = sz_decimals as usize);
		// Remove trailing zeros and decimal point if not needed, but only after the decimal point
		let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
		trimmed.to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clamp_size() {
		// Test with sz_decimals = 3 - truncates (rounds down)
		assert_eq!(clamp_size(1.125, 3), "1.125");
		assert_eq!(clamp_size(1.9999, 3), "1.999"); // Truncates to 1.999
		assert_eq!(clamp_size(1.0, 3), "1");

		// Test with sz_decimals = 0 (integer only) - truncates
		assert_eq!(clamp_size(1.9, 0), "1"); // Truncates to 1
		assert_eq!(clamp_size(1.4, 0), "1"); // Truncates to 1
		assert_eq!(clamp_size(10.0, 0), "10");
		assert_eq!(clamp_size(0.9, 0), "0"); // Truncates to 0

		// Test with sz_decimals = 2 - truncates
		assert_eq!(clamp_size(1.25, 2), "1.25");
		assert_eq!(clamp_size(1.999, 2), "1.99"); // Truncates to 1.99
		assert_eq!(clamp_size(0.01, 2), "0.01");
	}

	#[test]
	fn test_clamp_price_perp_basic() {
		// Perp: MAX_DECIMALS = 6, sz_decimals impacts max price decimals
		// With sz_decimals = 0, max price decimals = 6

		// Test 5 significant figures limit
		assert_eq!(clamp_price(1234.5, 0, false), "1234.5");
		assert_eq!(clamp_price(1234.56, 0, false), "1234.5"); // Truncates to 5 sig figs

		// Test small numbers
		assert_eq!(clamp_price(0.001234, 0, false), "0.001234"); // 4 sig figs, 6 decimals - OK
		assert_eq!(clamp_price(0.0012345, 0, false), "0.001234"); // 5 sig figs, 7 decimals - truncated to 6 decimals

		// Test integer prices (always allowed)
		assert_eq!(clamp_price(123456.0, 0, false), "123456");
		assert_eq!(clamp_price(1000000.0, 0, false), "1000000");
	}

	#[test]
	fn test_clamp_price_perp_with_sz_decimals() {
		// With sz_decimals = 2, max price decimals = 6 - 2 = 4

		// Test that max decimals constraint applies
		assert_eq!(clamp_price(1.123456, 2, false), "1.1234"); // Limited to 4 decimals, truncated
		assert_eq!(clamp_price(10.12345, 2, false), "10.123"); // 5 sig figs = 3 decimals after 10

		// Test with sz_decimals = 4, max price decimals = 2
		assert_eq!(clamp_price(1.234, 4, false), "1.23"); // Limited to 2 decimals
		assert_eq!(clamp_price(100.12, 4, false), "100.12"); // 5 sig figs (1,0,0,1,2) = 2 decimals, max 2 allowed
	}

	#[test]
	fn test_clamp_price_spot_basic() {
		// Spot: MAX_DECIMALS = 8, sz_decimals impacts max price decimals
		// With sz_decimals = 0, max price decimals = 8

		// Test 5 significant figures limit
		assert_eq!(clamp_price(1234.5, 0, true), "1234.5");
		assert_eq!(clamp_price(1234.56, 0, true), "1234.5"); // Truncates to 5 sig figs

		// Test small numbers - spot allows more decimals
		assert_eq!(clamp_price(0.00001234, 0, true), "0.00001234"); // 4 sig figs, 8 decimals - OK
		assert_eq!(clamp_price(0.000012345, 0, true), "0.00001234"); // 5 sig figs, 9 decimals - truncated to 8

		// Test integer prices (always allowed)
		assert_eq!(clamp_price(123456.0, 0, true), "123456");
	}

	#[test]
	fn test_clamp_price_spot_with_sz_decimals() {
		// With sz_decimals = 1, max price decimals = 8 - 1 = 7
		assert_eq!(clamp_price(0.0001234, 1, true), "0.0001234");
		assert_eq!(clamp_price(0.00012345, 1, true), "0.0001234"); // Truncates to 5 sig figs

		// With sz_decimals = 2, max price decimals = 8 - 2 = 6
		// This should be invalid per docs if sz_decimals > 2
		assert_eq!(clamp_price(0.0001234, 2, true), "0.000123"); // Limited to 6 decimals

		// With sz_decimals = 6, max price decimals = 2
		assert_eq!(clamp_price(1.234, 6, true), "1.23"); // Limited to 2 decimals
	}

	#[test]
	fn test_clamp_price_edge_cases() {
		// Test zero
		assert_eq!(clamp_price(0.0, 0, false), "0");

		// Test very small numbers
		assert_eq!(clamp_price(0.00000001, 0, false), "0.00000001"); // 1 sig fig, 8 decimals

		// Test truncation behavior
		assert_eq!(clamp_price(1.23456, 0, false), "1.2345"); // Truncates
		assert_eq!(clamp_price(1.23454, 0, false), "1.2345"); // Truncates

		// Test that integers don't get unnecessary decimals
		assert_eq!(clamp_price(100.0, 0, false), "100");
		assert_eq!(clamp_price(1.0, 3, false), "1");
	}

	#[test]
	fn test_clamp_price_sig_figs_vs_decimals() {
		// Test cases where significant figures limit comes into play

		// For 1234.5 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 4, so with 5 sig figs we can have 1 decimal
		// - Min(1, 6) = 1 decimal allowed
		assert_eq!(clamp_price(1234.56, 0, false), "1234.5"); // Truncates

		// For 12.345 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 2, so with 5 sig figs we can have 3 decimals
		// - Min(3, 6) = 3 decimals allowed
		assert_eq!(clamp_price(12.3456, 0, false), "12.345"); // Truncates

		// For 1.23456 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 1, so with 5 sig figs we can have 4 decimals
		// - Min(4, 6) = 4 decimals allowed
		assert_eq!(clamp_price(1.234567, 0, false), "1.2345"); // Truncates
	}

	#[test]
	fn test_clamp_price_small_numbers() {
		// Test prices less than 1
		// For perp: max_decimals = 6

		// 0.001234 has 4 sig figs, 6 decimals - OK
		assert_eq!(clamp_price(0.001234, 0, false), "0.001234");

		// 0.0012345 has 5 sig figs, 7 decimals - truncated to 6 decimals
		assert_eq!(clamp_price(0.0012345, 0, false), "0.001234");

		// 0.00123456 has 6 sig figs, should be truncated to 5 sig figs = 6 decimals
		assert_eq!(clamp_price(0.00123456, 0, false), "0.001234");

		// Very small number with leading zeros - these exceed 6 decimals for perp
		// 0.0000012345 has 5 sig figs, 10 decimals - clamped to 6 decimals = 0.000001
		assert_eq!(clamp_price(0.0000012345, 0, false), "0.000001");
		// 0.00000123456 has 6 sig figs, 11 decimals - clamped to 5 sig figs and 6 decimals
		assert_eq!(clamp_price(0.00000123456, 0, false), "0.000001");
	}

	#[test]
	fn test_real_world_examples() {
		// ETH price around 2000 with perp (sz_decimals typically 3)
		// Max price decimals = 6 - 3 = 3
		assert_eq!(clamp_price(2000.5, 3, false), "2000.5");
		assert_eq!(clamp_price(2000.12, 3, false), "2000.1"); // 5 sig figs = 1 decimal

		// BTC price around 50000 with perp (sz_decimals typically 4)
		// Max price decimals = 6 - 4 = 2
		assert_eq!(clamp_price(50000.0, 4, false), "50000");
		assert_eq!(clamp_price(50123.0, 4, false), "50123");
		assert_eq!(clamp_price(50123.45, 4, false), "50123"); // 5 sig figs exceeded

		// Small cap token at 0.001 on spot (sz_decimals 0)
		// Max price decimals = 8
		assert_eq!(clamp_price(0.001234, 0, true), "0.001234");
		assert_eq!(clamp_price(0.0012345, 0, true), "0.0012345");

		// USDC price (should be close to 1.0)
		assert_eq!(clamp_price(1.0, 2, true), "1");
		assert_eq!(clamp_price(0.9999, 2, true), "0.9999");
	}
}
