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
use executor_primitives::AccountId;
use heima_primitives::{Identity, IdentityString};
use jsonrpsee::{types::ErrorObject, RpcModule};
use log::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetNextIntentIdParams {
	pub email: String,
}

pub fn register_get_next_intent_id(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_getNextIntentId", |params, ctx, _| async move {
			match params.parse::<GetNextIntentIdParams>() {
				Ok(params) => {
					let account =
						Identity::Email(IdentityString::new(params.email.as_bytes().to_vec()))
							.to_omni_account();
					let intent_id =
						ctx.intent_id_store.get(&AccountId::from(account)).await.map_err(|e| {
							error!("Could not get IntentId from store: {:?}", e);
							<ErrorCode as Into<ErrorObject>>::into(ErrorCode::InternalError)
						})?;
					Ok::<u32, ErrorObject>(intent_id + 1)
				},
				Err(_) => Err(ErrorCode::ParseError.into()),
			}
		})
		.expect("Failed to register getIntentId method");
}
