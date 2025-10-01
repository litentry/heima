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

use crate::ErrorCode;
use crate::{methods::pumpx::common::PumpxRpcError, server::RpcContext};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::ToHexPrefixed, Web2IdentityType};
use heima_authentication::constants::CLIENT_ID_PUMPX;
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOmniAccountParams {
	pub user_id: String,
}

// Directly converts Identity to OmniAccount using 1:1 mapping
pub fn register_get_omni_account<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("pumpx_getOmniAccount", |params, _, _| async move {
			let params = params.parse::<GetOmniAccountParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			let account =
				Identity::from_web2_account(params.user_id.as_str(), Web2IdentityType::Pumpx)
					.to_omni_account(CLIENT_ID_PUMPX);
			Ok::<String, ErrorObject>(account.to_hex())
		})
		.expect("Failed to register pumpx_getOmniAccount method");
}
