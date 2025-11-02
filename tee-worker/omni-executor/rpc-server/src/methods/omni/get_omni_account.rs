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

use crate::detailed_error::DetailedError;
use crate::error_code::PARSE_ERROR_CODE;
use crate::methods::omni::common::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::hex_encode, Web2IdentityType};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOmniAccountParams {
	pub client_id: String,
	pub user_email: String,
}

// Directly converts Identity to OmniAccount using 1:1 mapping
pub fn register_get_omni_account<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_getOmniAccount", |params, _, _| async move {
			let params = params.parse::<GetOmniAccountParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
			})?;

			let account =
				Identity::from_web2_account(params.user_email.as_str(), Web2IdentityType::Email)
					.to_omni_account(&params.client_id);
			Ok::<String, ErrorObject>(hex_encode(account.as_ref()))
		})
		.expect("Failed to register omni_getOmniAccount method");
}
