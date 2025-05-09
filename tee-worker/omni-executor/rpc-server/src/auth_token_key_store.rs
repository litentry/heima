use executor_core::key_store::KeyStore;
use executor_crypto::rsa::RsaPrivateKey;
use rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey};

pub struct AuthTokenKeyStore {
	path: String,
}

impl AuthTokenKeyStore {
	pub fn new(path: String) -> Self {
		let store = Self { path: path.clone() };
		let std_path = std::path::Path::new(&path);
		if !std_path.exists() {
			let key: Vec<u8> = Self::generate_key().unwrap();
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

impl KeyStore<Vec<u8>> for AuthTokenKeyStore {
	fn generate_key() -> Result<Vec<u8>, ()> {
		let mut rng = rand::thread_rng();
		let rsa_private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| {
			tracing::log::error!("Failed to generate RSA key: {:?}", e);
		})?;
		let private_key = rsa_private_key.to_pkcs1_der().map_err(|e| {
			tracing::log::error!("Failed to encode RSA key: {:?}", e);
		})?;
		Ok(private_key.as_bytes().to_vec())
	}

	fn serialize(k: &Vec<u8>) -> Result<Vec<u8>, ()> {
		Ok(k.to_vec())
	}

	fn deserialize(sealed: Vec<u8>) -> Result<Vec<u8>, ()> {
		let private_key = RsaPrivateKey::from_pkcs1_der(&sealed).map_err(|e| {
			tracing::log::error!("Failed to decode RSA key: {:?}", e);
		})?;
		let public_key = private_key.to_public_key().to_pkcs1_der().map_err(|e| {
			tracing::log::error!("Failed to encode RSA key: {:?}", e);
		})?;
		tracing::log::info!("Auth token (JWT) RSA public_key: {:?}", public_key.as_bytes());

		Ok(sealed)
	}

	fn path(&self) -> String {
		self.path.clone()
	}
}
