use rsa::{RsaPrivateKey, RsaPublicKey};

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

impl Default for ShieldingKey {
	fn default() -> Self {
		let mut rng = rand::thread_rng();
		let key = RsaPrivateKey::new(&mut rng, 3072).expect("Failed to generate RSA key");
		Self { key }
	}
}
