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
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::AccountId;
use executor_storage::{IntentIdStorage, Storage};
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct GetNextIntentIdParams {
	// can be of ss58 or hex format
	pub omni_account: String,
}

#[tracing::instrument(skip(params, ctx), fields(
	omni_account = %params.omni_account
))]
async fn handle_get_next_intent_id_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: GetNextIntentIdParams,
	ctx: Arc<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> Result<u32, ErrorObjectOwned> {
	debug!("Processing omni_getNextIntentId request");

	let account = AccountId::from_str(&params.omni_account).map_err(|e| {
		error!("Could not parse AccountId: {:?}", e);
		<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InvalidParams)
	})?;

	let storage = IntentIdStorage::new(ctx.storage_db.clone());
	let intent_id = storage
		.get(&account)
		.map_err(|e| {
			error!("Could not get IntentId from store: {:?}", e);
			<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InternalError)
		})?
		.unwrap_or_default();

	Ok(intent_id + 1)
}

pub fn register_get_next_intent_id<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module
		.register_async_method("omni_getNextIntentId", |params, ctx, _| async move {
			let params = params.parse::<GetNextIntentIdParams>()?;
			handle_get_next_intent_id_request(params, ctx).await
		})
		.expect("Failed to register getIntentId method");
}
