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

use crate::server::RpcContext;
use crate::ErrorCode;
use executor_primitives::{utils::hex::ToHexPrefixed, Web2IdentityType};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetOmniAccountParams {
	pub user_uid: String,
}

// TODO: the omni-account needs to be read from AccountStore once we enable it
pub fn register_get_omni_account(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_getOmniAccount", |params, _, _| async move {
			match params.parse::<GetOmniAccountParams>() {
				Ok(params) => {
					let account = Identity::from_web2_account(
						params.user_uid.as_str(),
						Web2IdentityType::Pumpx,
					)
					.to_omni_account();
					Ok::<String, ErrorObject>(account.to_hex())
				},
				Err(_) => Err(ErrorCode::ParseError.into()),
			}
		})
		.expect("Failed to register pumpx_getOmniAccount method");
}
