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

use async_trait::async_trait;

#[async_trait]
pub trait WildmetaApi: Send + Sync {
	/// Verify if agent_address and main_address are linked on Hyperliquid
	/// login_type is passed to API for additional verification logic
	async fn verify_hyperliquid_link(
		&self,
		agent_address: &str,
		main_address: &str,
		login_type: u32,
	) -> Result<bool, ()>;
}

/// Mock implementation for testing
#[cfg(feature = "mocks")]
pub struct MockWildmetaApi;

#[cfg(feature = "mocks")]
#[async_trait]
impl WildmetaApi for MockWildmetaApi {
	async fn verify_hyperliquid_link(
		&self,
		_agent_address: &str,
		_main_address: &str,
		_login_type: u32,
	) -> Result<bool, ()> {
		// For testing, always return true
		// In production, this will call the actual Wildmeta API
		Ok(true)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_mock_wildmeta_api() {
		let api = MockWildmetaApi;
		let result = api.verify_hyperliquid_link("agent", "main", 1).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), true);
	}
}