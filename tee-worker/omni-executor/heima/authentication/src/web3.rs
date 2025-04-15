use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
	let timestep = now / period_in_seconds;
	format!("{:06}", timestep % 1_000_000)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn test_generate_message_code() {
		let period = 5;
		let code = generate_message_code(period);
		assert_eq!(code.len(), 6);

		// Generate the code within the same period
		sleep(Duration::new(2, 0));
		let expected_code = generate_message_code(period);

		assert_eq!(code, expected_code);

		// Simulate waiting for the period to pass
		sleep(Duration::new(4, 0));
		let new_code = generate_message_code(period);
		assert_ne!(code, new_code);
	}
}
