use executor_core::key_store::KeyStore;
use executor_crypto::aes256::Aes256Key;
use rand::Rng;

pub struct Aes256KeyStore {
	path: String,
}

impl Aes256KeyStore {
	pub fn new(path: String) -> Self {
		if std::path::Path::new(&path).exists() {
			Self { path }
		} else {
			let key = Self::generate_key().unwrap();
			let store = Self { path };
			store.write(&key).unwrap();
			store
		}
	}
}

impl KeyStore<Aes256Key> for Aes256KeyStore {
	fn generate_key() -> Result<Aes256Key, ()> {
		let mut rng = rand::thread_rng();
		Ok(rng.gen())
	}

	fn serialize(k: &Aes256Key) -> Result<Vec<u8>, ()> {
		Ok(k.to_vec())
	}

	fn deserialize(sealed: Vec<u8>) -> Result<Aes256Key, ()> {
		sealed.as_slice().try_into().map_err(|_| ())
	}

	fn path(&self) -> String {
		self.path.clone()
	}
}
