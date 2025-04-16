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

use std::str::FromStr;

use crate::server::RpcContext;
use crate::ErrorCode;
use executor_primitives::AccountId;
use executor_storage::{IntentIdStorage, Storage};
use jsonrpsee::{types::ErrorObject, RpcModule};
use log::error;
use parentchain_rpc_client::AccountId32;

pub fn register_get_next_intent_id(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getNextIntentId", |params, ctx, _| async move {
			//ss58 encoded account_id
			match params.parse::<String>() {
				Ok(omni_account) => {
					let account = AccountId32::from_str(&omni_account).map_err(|e| {
						error!("Could not parse AccountId: {:?}", e);
						<ErrorCode as Into<ErrorObject>>::into(ErrorCode::InvalidParams)
					})?;

					let storage = IntentIdStorage::new(ctx.storage_db.clone());
					let intent_id = storage.get(&AccountId::from(account.0)).ok_or_else(|| {
						log::error!("Could not get IntentId from store");
						<ErrorCode as Into<ErrorObject>>::into(ErrorCode::InternalError)
					})?;
					Ok::<u32, ErrorObject>(intent_id + 1)
				},
				Err(_) => Err(ErrorCode::ParseError.into()),
			}
		})
		.expect("Failed to register getIntentId method");
}
