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
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::submit_user_ops;
use crate::utils::validation::{parse_as, parse_rpc_params};
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::ChainId;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpTestParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	#[allow(dead_code)]
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpTestResponse {
	pub transaction_hash: Option<String>,
}

pub fn register_submit_user_op_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_submitUserOpTest", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<SubmitUserOpTestParams>(params)?;

			debug!("Received omni_submitUserOpTest, params: {:?}", params);

			for op in &params.user_operations {
				let _: Address = parse_as(&op.sender, "sender")?;
			}

			let omni_account = to_omni_account(&params.omni_account)?;

			let transaction_hash = submit_user_ops(
				&ctx,
				params.user_operations,
				params.chain_id,
				params.wallet_index,
				&omni_account,
			)
			.await?;

			Ok::<SubmitUserOpTestResponse, ErrorObjectOwned>(SubmitUserOpTestResponse {
				transaction_hash,
			})
		})
		.expect("Failed to register omni_submitUserOpTest method");
}
