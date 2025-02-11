use executor_crypto::{
	rsa::{errors::Error as RsaError, sha2::Sha256, Oaep, RsaPrivateKey, RsaPublicKey},
	traits::Decrypt,
};

#[derive(Debug, Clone)]
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

impl Decrypt for ShieldingKey {
	type Error = RsaError;

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
