use std::fmt::Debug;
use std::vec::Vec;

pub trait ShieldingCryptoEncrypt {
	type Error: Debug;
	fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

pub trait ShieldingCryptoDecrypt {
	type Error: Debug;
	fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
