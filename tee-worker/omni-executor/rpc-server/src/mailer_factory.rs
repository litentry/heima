// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use config_loader::{ConfigLoader, MailerConfig, MailerType};
use heima_identity_verification::web2::email::mailer::MailerTrait;
use heima_identity_verification::web2::email::{ConsoleMailer, Mailer};
use std::collections::HashMap;
use std::sync::Arc;

/// A factory for creating mailers based on client configurations
pub struct MailerFactory {
	config_loader: Arc<ConfigLoader>,
	// Cache of created mailers to avoid recreation
	mailer_cache: std::sync::RwLock<HashMap<String, Arc<dyn MailerTrait + Send + Sync>>>,
}

impl MailerFactory {
	pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
		Self { config_loader, mailer_cache: std::sync::RwLock::new(HashMap::new()) }
	}

	/// Get or create a mailer for the specified client
	pub fn get_mailer_for_client(
		&self,
		client_id: &str,
	) -> Result<Arc<dyn MailerTrait + Send + Sync>, Box<dyn std::error::Error>> {
		let client_key = client_id.to_lowercase();

		// Try to get from cache first
		{
			let cache =
				self.mailer_cache.read().map_err(|e| format!("Failed to read cache: {}", e))?;
			if let Some(mailer) = cache.get(&client_key) {
				return Ok(mailer.clone());
			}
		}

		// Get client-specific configuration
		let config = self
			.config_loader
			.get_mailer_config(client_id)
			.ok_or_else(|| format!("No mailer configuration found for client '{}'", client_id))?;

		let mailer = Self::create_mailer(config)?;

		// Cache the mailer
		{
			let mut cache =
				self.mailer_cache.write().map_err(|e| format!("Failed to write cache: {}", e))?;
			cache.insert(client_key.clone(), mailer.clone());
		}

		Ok(mailer)
	}

	/// Create a mailer instance from configuration
	fn create_mailer(
		config: MailerConfig,
	) -> Result<Arc<dyn MailerTrait + Send + Sync>, Box<dyn std::error::Error>> {
		let mailer: Arc<dyn MailerTrait + Send + Sync> = match config.mailer_type {
			MailerType::Console => {
				tracing::info!(
					"Creating Console Mailer - verification codes will be printed to logs"
				);
				Arc::new(ConsoleMailer::new())
			},
			MailerType::Sendgrid => {
				tracing::info!(
					"Creating SendGrid Mailer - verification codes will be sent via email from {}",
					config.mailer_from_email
				);
				Arc::new(Mailer::new(
					config.mailer_api_host,
					config.mailer_api_key,
					config.mailer_from_email,
					config.mailer_from_name,
				))
			},
		};

		Ok(mailer)
	}

	/// List all available client configurations
	pub fn list_available_clients(&self) -> Vec<String> {
		let mut clients: Vec<String> = self.config_loader.mailer_configs.keys().cloned().collect();
		clients.sort();
		clients
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use config_loader::ConfigLoader;

	#[test]
	fn test_mailer_factory_caching() {
		// Test that mailers are cached properly
		let config = ConfigLoader::from_env();
		let factory = MailerFactory::new(Arc::new(config));

		let mailer1 = factory.get_mailer_for_client("test_client").expect("Should create mailer");
		let mailer2 =
			factory.get_mailer_for_client("test_client").expect("Should get cached mailer");

		// Should be the same instance (Arc points to same object)
		assert!(Arc::ptr_eq(&mailer1, &mailer2));
	}
}
