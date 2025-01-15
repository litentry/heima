use crate::crypto::traits::ShieldingCryptoDecrypt;
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

pub struct ShieldingKey {
	key: RsaPrivateKey,
}

impl ShieldingKey {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn public_key(&self) -> RsaPublicKey {
		self.key.to_public_key()
	}

	pub fn private_key(&self) -> &RsaPrivateKey {
		&self.key
	}
}

impl ShieldingCryptoDecrypt for ShieldingKey {
	type Error = rsa::errors::Error;

	fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
		self.private_key().decrypt(Oaep::new::<Sha256>(), data)
	}
}

impl Default for ShieldingKey {
	fn default() -> Self {
		let mut rng = rand::thread_rng();
		let key = RsaPrivateKey::new(&mut rng, 3072).expect("Failed to generate RSA key");
		Self { key }
	}
}
