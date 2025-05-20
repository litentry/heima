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

use executor_primitives::utils::hex::FromHexPrefixed;
use heima_primitives::Address32;
use heima_primitives::IntentId;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1::EncodeRsaPublicKey;

use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::pumpx::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use ethers::types::Bytes;
use executor_core::native_task::NativeTask;
use executor_core::native_task::NativeTaskWrapper;
use executor_core::native_task::PumpxChainId;
use executor_core::native_task::PumxWalletIndex;
use executor_crypto::jwt;
use executor_primitives::OmniAuth;
use heima_authentication::auth_token::AuthTokenClaims;
use heima_primitives::Identity;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use rsa::RsaPrivateKey;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, error};

use super::common::handle_pumpx_native_task;

#[derive(Debug, Deserialize)]
pub struct SignLimitOrderParams {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub unsigned_tx: Vec<Bytes>,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct SignLimitOrderResponse {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub signed_tx: Vec<Bytes>,
}

pub fn register_sign_limit_order_params(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_signLimitOrder", |params, ctx, _| async move {
			let params = params.parse::<SignLimitOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received pumpx_signLimitOrder, intent_id: {}, order_id: {}, chain_id: {}, wallet_index: {}", params.intent_id, params.order_id, params.chain_id, params.wallet_index);

			let private_key =
				RsaPrivateKey::from_pkcs1_der(&ctx.jwt_rsa_private_key).map_err(|e| {
					error!("Failed to parse private key: {:?}", e);
					PumpxRpcError::from_error_code(ErrorCode::InternalError)
				})?;

			let public_key = private_key.to_public_key().to_pkcs1_der().map_err(|e| {
				error!("Failed to generate public key: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::InternalError)
			})?;

			// this validates jwt - we skip exp check for this call
			let Ok(token) =
				jwt::decode::<AuthTokenClaims>(&params.auth_token, public_key.as_bytes(), true)
			else {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			};
			if token.typ != "access" {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			let omni_account = token.sub;
			let Ok(address) = Address32::from_hex(&omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wrapper = NativeTaskWrapper {
				task: NativeTask::PumpxSignLimitOrder(
					Identity::Substrate(address),
					params.chain_id,
					params.wallet_index,
					params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect(),
				),
				nonce: None,
				auth: Some(OmniAuth::AuthToken(params.auth_token)),
			};

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxSignLimitOrder(signed_txs) => Ok(SignLimitOrderResponse {
					intent_id: params.intent_id,
					order_id: params.order_id,
					chain_id: params.chain_id,
					signed_tx: signed_txs.into_iter().map(Bytes::from).collect(),
				}),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_signLimitOrder method");
}
