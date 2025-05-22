use crate::key_store::KeyStore;
use executor_crypto::{ed25519, PairTrait};

pub struct Ed25519KeyStore {
	path: String,
}

impl Ed25519KeyStore {
	pub fn new(path: String) -> Self {
		let store = Self { path: path.clone() };
		let std_path = std::path::Path::new(&path);
		if !std_path.exists() {
			let key: [u8; 32] = Self::generate_key().unwrap();
			// Create store file if not exists.
			if let Some(parent) = std_path.parent() {
				if !parent.exists() {
					std::fs::create_dir_all(parent).unwrap();
				}
			}
			let _file = std::fs::File::create(&path).unwrap();
			store.write(&key).unwrap();
		}

		store
	}
}

impl KeyStore<[u8; 32]> for Ed25519KeyStore {
	fn generate_key() -> Result<[u8; 32], ()> {
		Ok(ed25519::Pair::generate().1)
	}

	fn serialize(k: &[u8; 32]) -> Result<Vec<u8>, ()> {
		Ok(Vec::from(k))
	}

	fn deserialize(sealed: Vec<u8>) -> Result<[u8; 32], ()> {
		sealed.as_slice().try_into().map_err(|_| ())
	}

	fn path(&self) -> String {
		self.path.clone()
	}
}
