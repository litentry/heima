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
use executor_primitives::AccountId;
use executor_storage::{IntentIdStorage, Storage};
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetNextIntentIdParams {
	pub user_id: String,
}

pub fn register_get_next_intent_id(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_getNextIntentId", |params, ctx, _| async move {
			let params = params.parse::<GetNextIntentIdParams>().map_err(|e| {
				log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			let account =
				Identity::from_web2_account(params.user_id.as_str(), Web2IdentityType::Pumpx)
					.to_omni_account();

			let storage = IntentIdStorage::new(ctx.storage_db.clone());
			let intent_id = storage.get(&AccountId::from(account)).ok_or_else(|| {
				log::error!("Could not get IntentId from store");
				PumpxRpcError::from_error_code(ErrorCode::InternalError)
			})?;
			Ok::<u32, ErrorObject>(intent_id + 1)
		})
		.expect("Failed to register getIntentId method");
}
