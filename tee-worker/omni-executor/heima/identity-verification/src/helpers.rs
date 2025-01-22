use rand::{thread_rng, Rng};
use std::string::String;

// This will be used for oauth2 authentication
#[allow(dead_code)]
pub(crate) fn generate_alphanumeric_otp(length: usize) -> String {
	let mut rng = thread_rng();
	let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
	let random_string: String = (0..length)
		.map(|_| {
			let idx = rng.gen_range(0..charset.len());
			charset[idx] as char
		})
		.collect();

	random_string
}

pub(crate) fn generate_otp(length: usize) -> String {
	let mut rng = rand::thread_rng();
	let otp: String = (0..length).map(|_| rng.gen_range(0..10).to_string()).collect();
	otp
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_random_string() {
		let random_string = generate_alphanumeric_otp(128);
		assert_eq!(random_string.len(), 128);
	}

	#[test]
	fn test_generate_otp() {
		let otp = generate_otp(6);
		assert_eq!(otp.len(), 6);
	}
}
