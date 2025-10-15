use config_loader::{AppleOAuth2Config, ConfigLoader};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AppleOAuth2Factory {
	config_loader: Arc<ConfigLoader>,
	config_cache: std::sync::RwLock<HashMap<String, AppleOAuth2Config>>,
}

impl AppleOAuth2Factory {
	pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
		Self { config_loader, config_cache: std::sync::RwLock::new(HashMap::new()) }
	}

	pub fn get_apple_config_for_client(
		&self,
		client_id: &str,
	) -> Result<AppleOAuth2Config, Box<dyn std::error::Error>> {
		let client_key = client_id.to_lowercase();

		let cache = self.config_cache.read().map_err(|e| format!("Failed to read cache: {}", e))?;
		if let Some(config) = cache.get(&client_key) {
			return Ok(config.clone());
		}

		drop(cache);

		let config = self.config_loader.get_apple_oauth2_config(client_id).ok_or_else(|| {
			let available_clients = self.config_loader.list_available_clients();
			format!(
				"No Apple OAuth2 configuration found for client '{}'. Available clients: {:?}",
				client_id, available_clients
			)
		})?;

		tracing::info!("Loaded Apple OAuth2 config for client '{}'", client_id);

		let mut cache =
			self.config_cache.write().map_err(|e| format!("Failed to write cache: {}", e))?;
		cache.insert(client_key.clone(), config.clone());

		Ok(config)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use config_loader::ConfigLoader;

	#[test]
	fn test_apple_oauth2_factory_caching() {
		std::env::set_var("OE_APPLE_CLIENT_ID_TESTCLIENT", "test_client_id");
		std::env::set_var("OE_APPLE_CLIENT_SECRET_TESTCLIENT", "test_client_secret");

		let config = ConfigLoader::from_env();
		let factory = AppleOAuth2Factory::new(Arc::new(config));

		let config1 =
			factory.get_apple_config_for_client("testclient").expect("Should create config");
		let config2 = factory
			.get_apple_config_for_client("testclient")
			.expect("Should get cached config");

		assert_eq!(config1.client_id, config2.client_id);
		assert_eq!(config1.client_secret, config2.client_secret);
	}

	#[test]
	fn test_apple_oauth2_factory_missing_client() {
		let config = ConfigLoader::from_env();
		let factory = AppleOAuth2Factory::new(Arc::new(config));

		let result = factory.get_apple_config_for_client("nonexistent");
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("No Apple OAuth2 configuration found"));
	}
}
