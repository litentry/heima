use crate::{
	types::{CallResponse, NativeOperationError},
	NativeOperationResponse, ResponseSender, TaskHandlerContext,
};
use executor_core::{intent_executor::IntentExecutor, native_operation::NativeCall};
use executor_crypto::{aes256::aes_encrypt_default, jwt};
use executor_primitives::{intent::Intent, MemberAccount, OmniAccountAuthType, ValidationData};
use executor_storage::{MemberOmniAccountStorage, Storage};
use heima_authentication::auth_token::{AuthOptions, AuthTokenClaims, AUTH_TOKEN_EXPIRATION};
use heima_identity_verification::{get_verification_message, web2, web3};
use parentchain_api_interface::runtime_types::{
	frame_system::pallet::Call as SystemCall,
	pallet_balances::pallet::Call as BalancesCall,
	pallet_omni_account::pallet::{Call as OmniAccountCall, IntentExecutionResult},
	paseo_runtime::RuntimeCall,
};
use parentchain_rpc_client::{
	AccountId32, SubstrateRpcClient, SubstrateRpcClientFactory, ToSubxtType, XtStatus,
};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;

pub async fn handle_native_call<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
	call: NativeCall,
	auth_type: OmniAccountAuthType,
	response_sender: ResponseSender,
) {
	let Ok(mut rpc_client) = ctx.parentchain_rpc_client_factory.new_client().await else {
		log::error!("Failed to create rpc client");
		let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
		if response_sender.send(response.encode()).is_err() {
			log::error!("Failed to send response");
		}
		return;
	};

	let (response_sender, tx) = match call {
		NativeCall::request_auth_token(sender_identity) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(current_block) = rpc_client.get_last_finalized_block_num().await else {
				log::error!("Failed to get last finalized block number");
				let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_options = AuthOptions { expires_at: current_block + AUTH_TOKEN_EXPIRATION };
			let login_claims = AuthTokenClaims::new(
				sender_identity.hash().to_string(),
				"login".to_string(),
				auth_options.clone(),
			);
			let trade_claims = AuthTokenClaims::new(
				sender_identity.hash().to_string(),
				"login".to_string(),
				auth_options,
			);
			let Ok(login_token) = jwt::create(&login_claims, &ctx.jwt_rsa_private_key) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(trade_token) = jwt::create(&trade_claims, &ctx.jwt_rsa_private_key) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), login_claims.exp);

			let tx = ctx.transaction_signer.sign(auth_token_requested_call, None).await;

			if rpc_client.submit_tx(&tx).await.is_err() {
				log::error!("Failed to submit tx");
				let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}

			let response: NativeOperationResponse =
				CallResponse::AuthToken { login_token, trade_token }.into();

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
		NativeCall::request_intent(sender_identity, intent) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let request_intent_call =
				OmniAccountCall::request_intent { intent: intent.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(request_intent_call),
					auth_type.to_subxt_type(),
				);

			let signer_account_id = ctx.transaction_signer.get_signer_account_id();
			let mut nonce = match rpc_client.get_account_nonce(&signer_account_id).await {
				Ok(n) => n,
				Err(e) => {
					log::error!("Failed to get account nonce: {:?}", e);
					let response =
						NativeOperationResponse::Err(NativeOperationError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				},
			};

			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await;
			if rpc_client.submit_tx(&tx).await.is_err() {
				log::error!("Failed to submit request_intent tx");
				let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}
			// Increment nonce for the next transaction
			nonce += 1;

			let mut execution_result = IntentExecutionResult::Success;

			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender_identity.hash().to_subxt_type(),
							RuntimeCall::System(remark_call),
							auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await
				},
				Intent::TransferNative(transfer) => {
					let transfer_call = BalancesCall::transfer_allow_death {
						dest: transfer.to.to_subxt_type().into(),
						value: transfer.value,
					};
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender_identity.hash().to_subxt_type(),
							RuntimeCall::Balances(transfer_call),
							auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await
				},
				Intent::CallEthereum(_) | Intent::TransferEthereum(_) => {
					if let Err(e) = ctx
						.ethereum_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::TransferSolana(_) => {
					if let Err(e) = ctx
						.solana_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::CrossChainSwap(_) => {
					if let Err(e) = ctx
						.cross_chain_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
			};

			(response_sender, tx)
		},
		NativeCall::create_account_store(sender_identity) => {
			let sender_identity_bytes = sender_identity.encode();
			let create_account_store_call = parentchain_api_interface::tx()
				.omni_account()
				.create_account_store(Decode::decode(&mut &sender_identity_bytes[..]).unwrap());
			let tx = ctx.transaction_signer.sign(create_account_store_call, None).await;
			(response_sender, tx)
		},
		NativeCall::add_account(
			sender_identity,
			identity,
			validation_data,
			public_account,
			permissions,
		) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(nonce) = rpc_client.get_account_nonce(&omni_account).await else {
				log::error!("Failed to get account nonce");
				let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let verification_message = get_verification_message(&sender_identity, &identity, nonce);

			let validation_result = match validation_data {
				ValidationData::Web2(web2_validation_data) => {
					if !identity.is_web2() {
						Err(NativeOperationError::InvalidMemberIdentity)
					} else {
						tokio::task::spawn_blocking({
							let identity = identity.clone();
							let storage_db = ctx.storage_db.clone();
							move || {
								web2::verify_identity(
									&identity,
									&verification_message,
									&web2_validation_data,
									storage_db,
								)
							}
						})
						.await
						.map_err(|e| {
							log::error!("Failed to verify identity: {:?}", e);
							NativeOperationError::InternalError
						})
						.and_then(|result| {
							result
								.map_err(|_| NativeOperationError::ValidationDataVerificationFailed)
						})
					}
				},
				ValidationData::Web3(web3_validation_data) => {
					if !identity.is_web3() {
						Err(NativeOperationError::InvalidMemberIdentity)
					} else {
						tokio::task::spawn_blocking({
							let identity = identity.clone();
							move || {
								web3::verify_identity(
									&identity,
									&verification_message,
									&web3_validation_data,
								)
							}
						})
						.await
						.map_err(|e| {
							log::error!("Failed to verify identity: {:?}", e);
							NativeOperationError::InternalError
						})
						.and_then(|result| {
							result
								.map_err(|_| NativeOperationError::ValidationDataVerificationFailed)
						})
					}
				},
			};
			if let Err(e) = validation_result {
				let response = NativeOperationResponse::Err(e);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}
			let member_account = match public_account {
				true => MemberAccount::Public(identity),
				false => MemberAccount::Private(
					aes_encrypt_default(&ctx.aes256_key, &identity.encode()).encode(),
					identity.hash(),
				),
			};
			let add_account_call = OmniAccountCall::add_account {
				member_account: member_account.to_subxt_type(),
				permissions: permissions.map(|p| p.to_subxt_type()),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(add_account_call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeCall::remove_accounts(sender_identity, identities) => {
			let remove_accounts = OmniAccountCall::remove_accounts {
				member_account_hashes: identities
					.iter()
					.map(|i| i.hash().to_subxt_type())
					.collect(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(remove_accounts),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeCall::publicize_account(sender_identity, identity) => {
			let publicize_account_call =
				OmniAccountCall::publicize_account { member_account: identity.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(publicize_account_call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeCall::set_permissions(sender_identity, identity, permissions) => {
			let set_permissions_call = OmniAccountCall::set_permissions {
				member_account_hash: identity.hash().to_subxt_type(),
				permissions: permissions.to_subxt_type(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(set_permissions_call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
	};
	let report = match rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await {
		Ok(report) => report,
		Err(e) => {
			log::error!("Failed to submit and watch tx: {:?}", e);
			let response = NativeOperationResponse::Err(NativeOperationError::InternalError);
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
	};
	let response: NativeOperationResponse = CallResponse::ExtrinsicReport {
		extrinsic_hash: report.extrinsic_hash,
		block_hash: report.block_hash,
		status: report.status,
	}
	.into();

	if response_sender.send(response.encode()).is_err() {
		log::error!("Failed to send response");
	}
}
