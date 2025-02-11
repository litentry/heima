use std::fmt::Debug;
use std::vec::Vec;

pub trait Encrypt {
	type Error: Debug;
	fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

pub trait Decrypt {
	type Error: Debug;
	fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
