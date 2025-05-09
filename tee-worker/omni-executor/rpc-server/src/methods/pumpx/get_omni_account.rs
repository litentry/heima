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
use executor_primitives::{utils::hex::ToHexPrefixed, Web2IdentityType};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOmniAccountParams {
	pub user_id: String,
}

// TODO: the omni-account needs to be read from AccountStore once we enable it
pub fn register_get_omni_account(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_getOmniAccount", |params, _, _| async move {
			let params = params.parse::<GetOmniAccountParams>().map_err(|e| {
				tracing::log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			let account =
				Identity::from_web2_account(params.user_id.as_str(), Web2IdentityType::Pumpx)
					.to_omni_account();
			Ok::<String, ErrorObject>(account.to_hex())
		})
		.expect("Failed to register pumpx_getOmniAccount method");
}
