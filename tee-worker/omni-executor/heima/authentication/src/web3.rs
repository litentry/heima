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
/// A tuple containing the generated code and the number of seconds left in the current period
pub fn generate_message_code(period_in_seconds: u64) -> (String, u64) {
	let now = Utc::now().timestamp() as u64;
	let code = message_code_at(period_in_seconds, now);
	let seconds_left = seconds_left_in_period(period_in_seconds, now);
	(code, seconds_left)
}

fn message_code_at(period: u64, timestamp: u64) -> String {
	let timestep = timestamp / period;
	format!("{:06}", timestep % 1_000_000)
}

fn seconds_left_in_period(period: u64, timestamp: u64) -> u64 {
	let elapsed = timestamp % period;
	period - elapsed
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_message_code_at() {
		let period = 5;

		// Testing period boundary behavior

		// Test same period (first period)
		assert_eq!(message_code_at(period, 0), message_code_at(period, 4));

		// Test period transitions
		assert_ne!(message_code_at(period, 4), message_code_at(period, 5));

		// Test with arbitrary timestamps in the same period
		let ts1 = 1000123; // Some arbitrary timestamp
		let period_start = (ts1 / period) * period; // Find the start of its period

		// All timestamps in the same period window should produce the same code
		for offset in 0..period {
			assert_eq!(
				message_code_at(period, period_start),
				message_code_at(period, period_start + offset)
			);
		}

		// Timestamps in the next period should produce a different code
		assert_ne!(
			message_code_at(period, period_start),
			message_code_at(period, period_start + period)
		);
	}

	#[test]
	fn test_seconds_left_in_period() {
		let period = 5;

		// Test with a timestamp that is exactly at the start of a period
		assert_eq!(seconds_left_in_period(period, 0), period);

		// Test with a timestamp that is halfway through a period
		assert_eq!(seconds_left_in_period(period, 2), 3);

		// Test with a timestamp that is just before the end of a period
		assert_eq!(seconds_left_in_period(period, 4), 1);

		// Test with a timestamp that is at the end of a period
		assert_eq!(seconds_left_in_period(period, 5), period);
	}
}
