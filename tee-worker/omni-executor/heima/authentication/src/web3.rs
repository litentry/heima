use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeimaMessagePayload {
	pub message_code: String,
}

pub const MESSAGE_CODE_PERIOD: u64 = 60; // 1 minute

/// Generates a time-based authentication code
///
/// This function creates a 6-digit code based on the current time divided by
/// the specified period, similar to TOTP (Time-based One-Time Password) algorithm.
///
/// # Arguments
///
/// * `period` - The time period in seconds for which the code remains valid
///
/// # Returns
///
/// A string containing a 6-digit message code that changes every `period` seconds
pub fn generate_message_code(period_in_seconds: u64) -> String {
	let now = Utc::now().timestamp() as u64;
	message_code_at(period_in_seconds, now)
}

fn message_code_at(period: u64, timestamp: u64) -> String {
	let timestep = timestamp / period;
	format!("{:06}", timestep % 1_000_000)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_message_code_at() {
		let period = 5;

		let base_ts = 1_000;

		let code1 = message_code_at(period, base_ts);
		// +2 seconds is still inside the same 5s window -> same code
		let code2 = message_code_at(period, base_ts + 2);
		assert_eq!(code1, code2);

		// +5 seconds jumps to the next window -> different code
		let code3 = message_code_at(period, base_ts + 5);
		assert_ne!(code1, code3);
	}
}
