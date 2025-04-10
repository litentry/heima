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

use crate::error_code::get_native_task_error_code;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
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
use jsonrpsee::types::ErrorObject;
use jsonrpsee::RpcModule;
use log::error;
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use parity_scale_codec::Decode;
use rsa::RsaPrivateKey;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;

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
			let internal_error: ErrorObject = ErrorCode::InternalError.into();

			let params: SignLimitOrderParams = params.parse::<SignLimitOrderParams>()?;
			let private_key = RsaPrivateKey::from_pkcs1_der(&ctx.jwt_rsa_private_key)
				.map_err(|_| internal_error.clone())?;
			let public_key =
				private_key.to_public_key().to_pkcs1_der().map_err(|_| internal_error.clone())?;
			// this validates jwt

			let Ok(token) =
				jwt::decode::<AuthTokenClaims>(&params.auth_token, public_key.as_bytes())
			else {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			};
			if token.typ != "access" {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let omni_account = token.sub;
			let Ok(address) = Address32::from_hex(&omni_account) else {
				error!("Not a valid address");
				return Err(internal_error);
			};

			let task_wrapper = NativeTaskWrapper {
				task: NativeTask::PumpxSignLimitOrder(
					Identity::Substrate(address),
					params.chain_id,
					params.wallet_index,
					params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect(),
				),
				nonce: None,
				auth: Some(OmniAuth::AuthToken(params.auth_token)),
			};

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((task_wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(internal_error);
			}

			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| internal_error.clone())?;
					match native_task_response {
						Ok(NativeTaskOk::PumpxSignLimitOrder(signed_txs)) => {
							Ok(SignLimitOrderResponse {
								intent_id: params.intent_id,
								order_id: params.order_id,
								chain_id: params.chain_id,
								signed_tx: signed_txs.into_iter().map(Bytes::from).collect(),
							})
						},
						Err(NativeTaskError::InternalError) => {
							log::error!("Internal error in native task");
							Err(internal_error)
						},
						Err(native_task_error) => {
							log::error!("Native task error: {:?}", native_task_error);
							Err(ErrorCode::ServerError(get_native_task_error_code(
								&native_task_error,
							))
							.into())
						},
						_ => {
							log::error!("Unexpected response type");
							Err(internal_error)
						},
					}
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(internal_error)
				},
			}
		})
		.expect("Failed to register pumpx_signLimitOrder method");
}
