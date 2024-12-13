#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_rand as rand;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use crate::alloc::string::String;
use rand::{thread_rng, Rng};

pub(crate) fn get_random_string(length: usize) -> String {
	let mut rng = thread_rng();
	let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
	let random_string: String = (0..length)
		.map(|_| {
			#[cfg(all(not(feature = "std"), feature = "sgx"))]
			let idx = rng.gen_range(0, charset.len());
			#[cfg(all(not(feature = "sgx"), feature = "std"))]
			let idx = rng.gen_range(0..charset.len());

			charset[idx] as char
		})
		.collect();

	random_string
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_random_string() {
		let random_string = get_random_string(128);
		assert_eq!(random_string.len(), 128);
	}
}
