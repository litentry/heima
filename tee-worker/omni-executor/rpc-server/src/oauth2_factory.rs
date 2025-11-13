use config_loader::{ConfigLoader, OAuth2Config};
use oe_primitives::OAuth2Provider;
use std::collections::HashMap;
use std::sync::Arc;

pub struct OAuth2ConfigFactory {
	config_loader: Arc<ConfigLoader>,
	config_cache: std::sync::RwLock<HashMap<(String, String), OAuth2Config>>, // (client_id, provider) -> config
}

impl OAuth2ConfigFactory {
	pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
		Self { config_loader, config_cache: std::sync::RwLock::new(HashMap::new()) }
	}

	pub fn get_config(
		&self,
		client_id: &str,
		provider: OAuth2Provider,
	) -> Result<OAuth2Config, Box<dyn std::error::Error>> {
		let client_key = client_id.to_lowercase();
		let provider_str = match provider {
			OAuth2Provider::Google => "google",
			OAuth2Provider::Apple => "apple",
		};
		let cache_key = (client_key.clone(), provider_str.to_string());

		let cache = self.config_cache.read().map_err(|e| format!("Failed to read cache: {}", e))?;
		if let Some(config) = cache.get(&cache_key) {
			return Ok(config.clone());
		}

		drop(cache);

		let config =
			self.config_loader.get_oauth2_config(client_id, provider_str).ok_or_else(|| {
				let available_clients = self.config_loader.list_available_clients();
				format!(
					"No {} OAuth2 configuration found for client '{}'. Available clients: {:?}",
					provider_str, client_id, available_clients
				)
			})?;

		tracing::info!("Loaded {} OAuth2 config for client '{}'", provider_str, client_id);

		let mut cache =
			self.config_cache.write().map_err(|e| format!("Failed to write cache: {}", e))?;
		cache.insert(cache_key, config.clone());

		Ok(config)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use config_loader::ConfigLoader;
	use oe_primitives::OAuth2Provider;

	#[test]
	fn test_oauth2_factory_caching_google() {
		std::env::set_var("OE_GOOGLE_CLIENT_ID_TESTCLIENT", "test_google_client_id");
		std::env::set_var("OE_GOOGLE_CLIENT_SECRET_TESTCLIENT", "test_google_secret");

		let config = ConfigLoader::from_env();
		let factory = OAuth2ConfigFactory::new(Arc::new(config));

		let config1 = factory
			.get_config("testclient", OAuth2Provider::Google)
			.expect("Should create config");
		let config2 = factory
			.get_config("testclient", OAuth2Provider::Google)
			.expect("Should get cached config");

		assert_eq!(config1.client_id, config2.client_id);
		assert_eq!(config1.client_secret, config2.client_secret);
	}

	#[test]
	fn test_oauth2_factory_caching_apple() {
		std::env::set_var("OE_APPLE_CLIENT_ID_TESTCLIENT", "test_apple_client_id");
		std::env::set_var("OE_APPLE_CLIENT_SECRET_TESTCLIENT", "test_apple_secret");

		let config = ConfigLoader::from_env();
		let factory = OAuth2ConfigFactory::new(Arc::new(config));

		let config1 = factory
			.get_config("testclient", OAuth2Provider::Apple)
			.expect("Should create config");
		let config2 = factory
			.get_config("testclient", OAuth2Provider::Apple)
			.expect("Should get cached config");

		assert_eq!(config1.client_id, config2.client_id);
		assert_eq!(config1.client_secret, config2.client_secret);
	}

	#[test]
	fn test_oauth2_factory_missing_client() {
		let config = ConfigLoader::from_env();
		let factory = OAuth2ConfigFactory::new(Arc::new(config));

		let result = factory.get_config("nonexistent", OAuth2Provider::Google);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("No google OAuth2 configuration found"));
	}
}
