mod aes256_key_store;
mod types;

use aa_contracts_client::calculate_user_operation_hash;
use aa_contracts_client::EntryPointClient;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use chrono::{Days, Utc};
use ethereum_rpc::AlloyRpcProvider;
use executor_core::{
	intent_executor::IntentExecutor,
	native_task::{NativeTask, NativeTaskWrapper},
	types::SerializablePackedUserOperation,
};
use executor_crypto::{
	aes256::{aes_decrypt, Aes256Key},
	jwt,
};
use executor_primitives::{
	utils::hex::{decode_hex, ToHexPrefixed},
	AccountId, ChainId, Identity, Intent, IntentId, OmniAccountAuthType, PumpxAccountProfile,
	Web2IdentityType,
};
use executor_storage::{HeimaJwtStorage, IntentIdStorage, PumpxProfileStorage, Storage, StorageDB};
use heima_authentication::{
	auth_token::*,
	constants::{AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE},
};
use parentchain_api_interface::runtime_types::{
	frame_system::pallet::Call as SystemCall, pallet_balances::pallet::Call as BalancesCall,
	pallet_omni_account::pallet::IntentCompletedDetail, paseo_runtime::RuntimeCall,
};
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	AccountId32, CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient,
	SubxtClientFactory, ToSubxtType,
};
use parentchain_signer::TxSigner;
use parity_scale_codec::Encode;
use pumpx::{
	methods::create_transfer_tx::CreateTransferTxBody, signer_client::PumpxChainId, PumpxApi,
};
use signer_client::{ChainType, SignerClient};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot, Semaphore};
use tracing::{debug, error, info, span, Instrument, Level};

pub use aes256_key_store::Aes256KeyStore;
pub use types::{NativeTaskError, NativeTaskOk, PumpxApiError, PumpxSignerError};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskChannelType = (NativeTaskWrapper<NativeTask>, ResponseSender);
pub type NativeTaskSender = mpsc::Sender<NativeTaskChannelType>;

pub type NativeTaskResponse = Result<NativeTaskOk, NativeTaskError>;

pub const MAX_CONCURRENT_TASKS: usize = 512; // TODO: make it configurable (if we go for semaphore)

pub type ParentchainTxSigner = TxSigner<
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

pub struct TaskHandlerContext<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	EthereumIntentExecutor: IntentExecutor,
	SolanaIntentExecutor: IntentExecutor,
	CrossChainIntentExecutor: IntentExecutor,
> {
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_rsa_private_key: Vec<u8>,
	pub aes256_key: Aes256Key,
	pub transaction_signer: Arc<ParentchainTxSigner>,
	pub ethereum_intent_executor: Arc<EthereumIntentExecutor>,
	pub solana_intent_executor: Arc<SolanaIntentExecutor>,
	pub cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	pub pumpx_api: Arc<Box<dyn PumpxApi>>,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pub entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
	pub rpc_clients: Arc<HashMap<u64, Arc<AlloyRpcProvider>>>,
	phantom_header: PhantomData<Header>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		Header,
		RpcClient: SubstrateRpcClient<Header>,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
		EthereumIntentExecutor: IntentExecutor,
		SolanaIntentExecutor: IntentExecutor,
		CrossChainIntentExecutor: IntentExecutor,
	>
	TaskHandlerContext<
		Header,
		RpcClient,
		RpcClientFactory,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		CrossChainIntentExecutor,
	>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
		storage_db: Arc<StorageDB>,
		jwt_rsa_private_key: Vec<u8>,
		aes256_key: Aes256Key,
		ethereum_intent_executor: Arc<EthereumIntentExecutor>,
		solana_intent_executor: Arc<SolanaIntentExecutor>,
		cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
		rpc_clients: Arc<HashMap<u64, Arc<AlloyRpcProvider>>>,
	) -> Self {
		Self {
			parentchain_rpc_client_factory,
			transaction_signer,
			storage_db,
			jwt_rsa_private_key,
			aes256_key,
			ethereum_intent_executor,
			solana_intent_executor,
			cross_chain_intent_executor,
			pumpx_api,
			pumpx_signer_client,
			entry_point_clients,
			rpc_clients,
			phantom_header: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}

	/// Get EntryPoint client for a specific chain
	pub fn get_entry_point_client(
		&self,
		chain_id: ChainId,
	) -> Option<Arc<EntryPointClient<AlloyRpcProvider>>> {
		self.entry_point_clients.get(&chain_id).cloned()
	}
}

pub async fn run_native_task_handler<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	buffer: usize,
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
) -> NativeTaskSender {
	// TODO: maybe not using a handler at all is better/simpler, jsonrpsee handles the method async already
	let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
	let (sender, mut receiver) = mpsc::channel::<NativeTaskChannelType>(buffer);

	tokio::spawn(async move {
		while let Some((wrapper, sender)) = receiver.recv().await {
			if let Ok(permit) = semaphore.clone().acquire_owned().await {
				let ctx_cloned = ctx.clone();
				tokio::spawn(async move {
					let span = span!(Level::INFO, "native-task", id = wrapper.id);
					let _permit = permit; // dropped when task finishes
					handle_native_task(ctx_cloned, wrapper, sender).instrument(span).await
				});
			}
		}
	});

	sender
}

async fn handle_native_task<
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
	wrapper: NativeTaskWrapper<NativeTask>,
	response_sender: ResponseSender,
) {
	let Ok(mut rpc_client) = ctx.parentchain_rpc_client_factory.new_client().await else {
		send_error(
			"Failed to create rpc client".to_string(),
			response_sender,
			NativeTaskError::InternalError,
		);
		return;
	};

	let auth_type: Option<OmniAccountAuthType> = wrapper.auth.map(|t| t.into());
	let client_id = &wrapper.client_id;

	match wrapper.task {
		NativeTask::RequestAuthToken(sender) => {
			// Convert Identity to AccountId directly using client_id
			let omni_account = sender.to_omni_account(client_id);
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };
			let claims = match sender {
				Identity::Email(ref identity_string) => {
					let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
						send_error(
							"Invalid email identity".to_string(),
							response_sender,
							NativeTaskError::InvalidMemberIdentity,
						);
						return;
					};
					AuthTokenClaims::new(
						email.to_string(),
						AUTH_TOKEN_ID_TYPE.to_string(),
						client_id.to_string(),
						auth_options,
					)
				},
				_ => AuthTokenClaims::new(
					sender.hash().to_string(),
					AUTH_TOKEN_ID_TYPE.to_string(),
					client_id.to_string(),
					auth_options,
				),
			};
			let Ok(token) = jwt::create(&claims, &ctx.jwt_rsa_private_key) else {
				send_error(
					"Failed to create auth token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			let tx = ctx.transaction_signer.sign(auth_token_requested_call).await;

			if rpc_client.submit_tx(&tx).await.is_err() {
				send_error(
					"Failed to submit tx".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				ctx.transaction_signer.update_nonce().await;
				return;
			}

			send_ok(response_sender, NativeTaskOk::AuthToken(token));
		},
		NativeTask::RequestIntent(omni_account, intent_id, intent) => {
			let intent = *intent; // Unbox the intent
			debug!("Intent requested");

			let intent_id_storage = IntentIdStorage::new(ctx.storage_db.clone());
			let stored_intent_id = match intent_id_storage.get(&omni_account) {
				Ok(id) => id.unwrap_or_default(),
				Err(_) => {
					send_error(
						"Failed to read intent from store".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				},
			};

			if intent_id == stored_intent_id + 1 {
				if intent_id_storage.insert(&omni_account, intent_id).is_err() {
					send_error(
						"Failed to save intent id".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				}
			} else {
				send_error(
					format!(
						"Intent id different than expected, expected: {:?}, got: {:?}",
						stored_intent_id + 1,
						intent_id
					),
					response_sender,
					NativeTaskError::IntentNonceMismatch,
				);
				return;
			}

			let _ = notify_intent_accepted(
				&mut rpc_client,
				ctx.transaction_signer.clone(),
				omni_account.clone(),
				intent_id,
				intent.clone(),
			)
			.await;

			let (execution_result, should_notify_parentchain) = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let _ = dispatch_as_signed(
						&mut rpc_client,
						ctx.transaction_signer.clone(),
						omni_account.clone(),
						RuntimeCall::System(remark_call),
						auth_type,
					)
					.await;
					send_ok(
						response_sender,
						NativeTaskOk::RequestIntentResult { intent_id, success: true },
					);
					(IntentCompletedDetail::Success, true)
				},
				Intent::TransferNative(transfer) => {
					let transfer_call = BalancesCall::transfer_allow_death {
						dest: transfer.to.to_subxt_type().into(),
						value: transfer.value,
					};
					let _ = dispatch_as_signed(
						&mut rpc_client,
						ctx.transaction_signer.clone(),
						omni_account.clone(),
						RuntimeCall::Balances(transfer_call),
						auth_type,
					)
					.await;
					send_ok(
						response_sender,
						NativeTaskOk::RequestIntentResult { intent_id, success: true },
					);
					(IntentCompletedDetail::Success, true)
				},
				Intent::CallEthereum(_) | Intent::TransferEthereum(_) => {
					// if let Err(e) = ctx
					// 	.ethereum_intent_executor
					// 	.execute(&omni_account, intent_id, intent.clone())
					// 	.await
					// {
					// 	log::error!("Error executing intent: {:?}", e);
					// 	send_ok(
					// 		response_sender,
					// 		NativeTaskOk::RequestIntentResult { intent_id, success: false },
					// 	);
					// 	(IntentCompletedDetail::Failure, true)
					// } else {
					// 	send_ok(
					// 		response_sender,
					// 		NativeTaskOk::RequestIntentResult { intent_id, success: true },
					// 	);
					// 	(IntentCompletedDetail::Success, true)
					// }
					info!("Intent rejected");
					send_error(
						"Intent not accepted".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					(IntentCompletedDetail::Failure, true)
				},
				Intent::TransferSolana(_) => {
					// if let Err(e) = ctx
					// 	.solana_intent_executor
					// 	.execute(&omni_account, intent_id, intent.clone())
					// 	.await
					// {
					// 	log::error!("Error executing intent: {:?}", e);
					// 	send_ok(
					// 		response_sender,
					// 		NativeTaskOk::RequestIntentResult { intent_id, success: false },
					// 	);
					// 	(IntentCompletedDetail::Failure, true)
					// } else {
					// 	send_ok(
					// 		response_sender,
					// 		NativeTaskOk::RequestIntentResult { intent_id, success: true },
					// 	);
					// 	(IntentCompletedDetail::Success, true)
					// }
					send_error(
						"Intent not accepted".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					(IntentCompletedDetail::Failure, true)
				},
				Intent::Swap(..) => {
					let (execution_result, should_notify_parentchain, response) = match ctx
						.cross_chain_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						Ok((response, should_notify_parentchain)) => {
							(IntentCompletedDetail::Success, should_notify_parentchain, response)
						},
						Err(e) => {
							error!("Error executing intent: {:?}", e);
							ctx.cross_chain_intent_executor.on_execution_error().await;
							(IntentCompletedDetail::Failure, true, None)
						},
					};
					if let Some(response) = response {
						send_ok(response_sender, NativeTaskOk::IntentSwapResponse(response));
					}
					(execution_result, should_notify_parentchain)
				},
			};

			if should_notify_parentchain {
				let _ = notify_intent_completed(
					&mut rpc_client,
					ctx.transaction_signer.clone(),
					omni_account.clone(),
					intent_id,
					execution_result,
				)
				.await;
			}
		},
		NativeTask::PumpxRequestJwt(_sender, email, invite_code, google_code, language) => {
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };

			debug!("Calling pumpx get_account_user_id, email: {}", email);
			let res = match ctx.pumpx_api.get_account_user_id(email.clone()).await {
				Ok(res) => res,
				Err(e) => {
					error!("Failed to get_account_user_id for email {}: {:?}", email, e);
					send_error(
						format!("Failed to get_account_user_id: {:?}", e),
						response_sender,
						NativeTaskError::PumpxApiError(PumpxApiError::GetAccountUserIdFailed),
					);
					return;
				},
			};
			debug!("Response pumpx get_account_user_id: {:?}", res);

			let Some(user_id) = res.data.user_id else {
				send_error(
					"Response data.user_id of call get_account_user_id is none".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GetAccountUserIdFailed),
				);
				return;
			};

			debug!("get_account_user_id ok, email: {}, user_id: {}", email, user_id);
			let omni_account = Identity::from_web2_account(&user_id, Web2IdentityType::Pumpx)
				.to_omni_account(client_id);

			let access_token_claims = AuthTokenClaims::new(
				omni_account.to_hex(),
				AUTH_TOKEN_ACCESS_TYPE.to_string(),
				client_id.to_string(),
				auth_options.clone(),
			);
			let Ok(access_token) = jwt::create(&access_token_claims, &ctx.jwt_rsa_private_key)
			else {
				send_error(
					"Failed to create access token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
				return;
			};

			debug!("Calling pumpx user_connect, user_id: {}, email: {}, invite_code: {:?}, google_code: {:?}", user_id, email, invite_code, google_code);
			let Ok(backend_response) = ctx
				.pumpx_api
				.user_connect(
					&access_token,
					user_id.clone(),
					email.clone(),
					invite_code,
					google_code,
					language,
				)
				.await
			else {
				send_error(
					"Failed to connect user".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::UserConnectionFailed),
				);
				return;
			};
			debug!("Response pumpx user_connect: {:?}", backend_response);

			// check google auth value
			if !backend_response.data.google_auth_check.unwrap_or(false) {
				send_error(
					"Google code verification failed from user_connect".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GoogleCodeVerificationFailed),
				);
				return;
			}

			let id_token_claims = AuthTokenClaims::new(
				omni_account.to_hex(),
				AUTH_TOKEN_ID_TYPE.to_string(),
				client_id.to_string(),
				auth_options,
			);
			let Ok(id_token) = jwt::create(&id_token_claims, &ctx.jwt_rsa_private_key) else {
				send_error(
					"Failed to create id token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
				return;
			};

			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			if storage
				.insert(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
				.is_err()
			{
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ACCESS_TYPE);
			};

			if storage.insert(&(omni_account, AUTH_TOKEN_ID_TYPE), id_token.clone()).is_err() {
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ID_TYPE);
			};

			send_ok(
				response_sender,
				NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response },
			);
		},
		NativeTask::PumpxExportWallet(
			omni_account,
			google_code,
			pumpx_chain_id,
			pumpx_wallet_index,
			expected_wallet_address,
		) => {
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					format!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			let verify_success = verify_google_code(
				ctx.pumpx_api.as_ref().as_ref(),
				&access_token,
				google_code,
				None,
			)
			.await;
			if !verify_success {
				send_error(
					"Failed to verify google code within NativeTask::PumpxExportWallet".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GoogleCodeVerificationFailed),
				);
				return;
			}

			let Some(chain) = ChainType::from_pumpx_chain_id(pumpx_chain_id) else {
				send_error(
					format!("Failed to map pumpx chain_id {}", pumpx_chain_id),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			let Ok(mut wallet) = ctx
				.pumpx_signer_client
				.export_wallet(
					chain,
					pumpx_wallet_index,
					omni_account.clone().into(),
					// TODO: theoretically we could pass the aes_key from initial RPC to signer, so that
					//       we don't have to do double encryption/decryption
					ctx.aes256_key.to_vec(),
					expected_wallet_address,
				)
				.await
			else {
				send_error(
					"Failed to export wallet from pumpx-signer".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};
			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				send_error(
					"No wallet after decryption".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			let omni_account_profile_storage = PumpxProfileStorage::new(ctx.storage_db.clone());
			if let Ok(maybe_profile) = omni_account_profile_storage.get(&omni_account) {
				let profile = maybe_profile
					.map(|mut p| {
						p.wallet_exported = true;
						p
					})
					.unwrap_or_else(|| PumpxAccountProfile { wallet_exported: true });
				if let Err(e) = omni_account_profile_storage.insert(&omni_account, profile) {
					error!("Failed to update pumpx account profile: {:?}", e);
					send_error(
						"Failed to update omni account profile".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				};
			} else {
				send_error(
					"Failed to get pumpx account profile".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			}
			send_ok(response_sender, NativeTaskOk::PumpxExportWallet(decrypted_wallet));
		},
		NativeTask::PumpxAddWallet(omni_account) => {
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account, AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					format!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			// Call Pumpx API to add wallet
			debug!("Calling pumpx add_wallet");
			let Ok(backend_response) = ctx.pumpx_api.add_wallet(&access_token, None).await else {
				send_error(
					"Failed to add wallet through Pumpx API".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::AddWalletFailed),
				);
				return;
			};

			send_ok(response_sender, NativeTaskOk::PumpxAddWallet(backend_response));
		},
		NativeTask::PumpxSignLimitOrder(omni_account, chain_id, wallet_index, unsigned_tx) => {
			let Some(chain) = ChainType::from_pumpx_chain_id(chain_id) else {
				error!("Failed to map pumpx chain_id {}", chain_id);
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					error!("Failed to send response");
				}
				return;
			};
			let Ok(signed_txs) = ctx
				.pumpx_signer_client
				.request_signatures(chain, wallet_index, omni_account.into(), unsigned_tx)
				.await
			else {
				error!("Failed to request signatures from pumpx-signer");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					error!("Failed to send response");
				}
				return;
			};
			let response = NativeTaskResponse::Ok(NativeTaskOk::PumpxSignLimitOrder(signed_txs));
			if response_sender.send(response.encode()).is_err() {
				error!("Failed to send response");
			}
		},
		NativeTask::PumpxTransferWidthdraw(
			omni_account,
			request_id,
			chain_id,
			wallet_index,
			recipient_address,
			token_ca,
			amount,
			google_code,
			language,
		) => {
			// 1. Verify we have a valid Pumpx "access" token for the user
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					"Failed to get access_token within NativeTask::PumpxTransferWidthdraw"
						.to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			// 2. Verify google code in every case
			let verify_success = verify_google_code(
				ctx.pumpx_api.as_ref().as_ref(),
				&access_token,
				google_code,
				language.clone(),
			)
			.await;
			if !verify_success {
				send_error(
					"Failed to verify google code within NativeTask::PumpxTransferWidthdraw"
						.to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GoogleCodeVerificationFailed),
				);
				return;
			}

			// 3. Create a transfer tx and send to backend
			let body = CreateTransferTxBody {
				request_id,
				chain_id,
				wallet_index,
				recipient_address,
				token_ca,
				amount,
			};

			debug!("Calling pumpx create_transfer_tx, body {:?}", body);
			match ctx.pumpx_api.create_transfer_tx(&access_token, body, language.clone()).await {
				Ok(res) => {
					send_ok(response_sender, NativeTaskOk::PumpxTransferWithdraw(res));
				},
				Err(e) => {
					send_error(
						format!("Failed to create_transfer_tx: {:?}", e),
						response_sender,
						NativeTaskError::PumpxApiError(PumpxApiError::CreateTransferTxFailed),
					);
				},
			};
		},
		NativeTask::PumpxNotifyLimitOrderResult(omni_account, intent_id, result, message) => {
			if result != "ok" && result != "nok" {
				send_error(
					format!("Invalid result value: {}. Must be 'ok' or 'nok'", result),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::InvalidInput),
				);
				return;
			}

			let execution_result = match result.as_str() {
				"ok" => IntentCompletedDetail::Success,
				"nok" => IntentCompletedDetail::Failure,
				_ => unreachable!(), // Already validated above
			};

			if let Some(msg) = message {
				info!("Limit order result message for intent_id {}: {}", intent_id, msg);
			}

			send_ok(response_sender, NativeTaskOk::PumpxNotifyLimitOrderResult);

			notify_intent_completed(
				&mut rpc_client,
				ctx.transaction_signer.clone(),
				omni_account,
				intent_id,
				execution_result,
			)
			.await;
		},
		NativeTask::SubmitUserOp(omni_account, serializable_user_ops, chain_id, wallet_index) => {
			info!(
				"Processing SubmitUserOp for {} UserOperations on chain_id: {}",
				serializable_user_ops.len(),
				chain_id
			);

			// Get EntryPoint client for this chain (needed for both signing and submission)
			let entry_point_client = match ctx.get_entry_point_client(chain_id) {
				Some(client) => client,
				None => {
					send_error(
						format!("No EntryPoint client configured for chain_id: {}", chain_id),
						response_sender,
						NativeTaskError::UnsupportedChain,
					);
					return;
				},
			};

			// Process each UserOperation in the batch
			let mut aa_user_ops = Vec::new();

			for (index, serializable_user_op) in serializable_user_ops.iter().enumerate() {
				// Convert SerializablePackedUserOperation to PackedUserOperation
				let mut packed_user_op =
					match convert_to_packed_user_op(serializable_user_op.clone()) {
						Ok(user_op) => user_op,
						Err(e) => {
							send_error(
								format!("Failed to convert UserOperation {}: {}", index, e),
								response_sender,
								NativeTaskError::InternalError,
							);
							return;
						},
					};

				// Check if UserOperation is signed
				if packed_user_op.signature.is_empty() {
					info!(
						"UserOperation {} is unsigned, requesting signature from pumpx signer",
						index
					);

					// Log UserOp details for debugging
					info!(
						"UserOp details - Sender: {}, Nonce: {}, InitCode length: {}, CallData length: {}",
						packed_user_op.sender,
						packed_user_op.nonce,
						packed_user_op.initCode.len(),
						packed_user_op.callData.len()
					);

					let entry_point_address = entry_point_client.entry_point_address();

					let user_op_hash_bytes = calculate_user_operation_hash(
						&packed_user_op,
						entry_point_address,
						chain_id,
					);
					let message_to_sign = user_op_hash_bytes.to_vec();

					info!(
						"Signing UserOp hash: 0x{}, EntryPoint: {}, ChainID: {}",
						hex::encode(&user_op_hash_bytes),
						entry_point_address,
						chain_id
					);

					// Request signature from pumpx signer for EVM chain
					let signature_result = ctx
						.pumpx_signer_client
						.request_signature(
							ChainType::Evm,
							wallet_index,
							omni_account.clone().into(),
							message_to_sign,
						)
						.await;

					let signature = match signature_result {
						Ok(sig) => sig,
						Err(_) => {
							send_error(
								format!("Failed to sign user operation {}", index),
								response_sender,
								NativeTaskError::PumpxSignerError(
									PumpxSignerError::RequestSignatureFailed,
								),
							);
							return;
						},
					};

					// Prepend 0x01 byte to indicate Root signature type (according to UserOpSigner enum)
					let mut signature_with_prefix: Vec<u8> = vec![0x01];
					signature_with_prefix.extend_from_slice(&signature);
					packed_user_op.signature = Bytes::from(signature_with_prefix);
					info!("UserOperation {} signed successfully", index);
				}

				// Convert to aa_contracts_client::PackedUserOperation for EntryPoint call
				let aa_user_op = aa_contracts_client::PackedUserOperation {
					sender: packed_user_op.sender,
					nonce: packed_user_op.nonce,
					initCode: packed_user_op.initCode.clone(),
					callData: packed_user_op.callData.clone(),
					accountGasLimits: packed_user_op.accountGasLimits,
					preVerificationGas: packed_user_op.preVerificationGas,
					gasFees: packed_user_op.gasFees,
					paymasterAndData: packed_user_op.paymasterAndData.clone(),
					signature: packed_user_op.signature.clone(),
				};
				aa_user_ops.push(aa_user_op);
			}

			// Get beneficiary address from the EntryPoint client's wallet
			let beneficiary = match entry_point_client.get_wallet_address().await {
				Ok(address) => address,
				Err(_) => {
					send_error(
						"Failed to get wallet address from EntryPoint client".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				},
			};

			// Run simulation for each UserOperation before submission
			for (index, aa_user_op) in aa_user_ops.iter().enumerate() {
				info!("Running simulation for UserOperation {}", index);
				match entry_point_client.simulate_validation(aa_user_op.clone()).await {
					Ok(validation_result) => {
						info!(
							"UserOperation {} simulation successful. PreOpGas: {}, Prefund: {}",
							index,
							validation_result.returnInfo.preOpGas,
							validation_result.returnInfo.prefund
						);
					},
					Err(_) => {
						send_error(
							format!("UserOperation {} simulation failed", index),
							response_sender,
							NativeTaskError::InternalError,
						);
						return;
					},
				}
			}
			info!("All UserOperations passed simulation checks");

			// Submit all UserOperations via EntryPoint.handleOps()
			let transaction_hash =
				match entry_point_client.handle_ops(&aa_user_ops, beneficiary).await {
					Ok(tx_hash) => {
						// Return the actual transaction hash from handle_ops
						Some(tx_hash)
					},
					Err(_) => {
						send_error(
							"Failed to submit UserOperations to EntryPoint via handleOps"
								.to_string(),
							response_sender,
							NativeTaskError::InternalError,
						);
						return;
					},
				};

			send_ok(response_sender, NativeTaskOk::SubmitUserOp(transaction_hash));
		},
	};
}

fn send_response(sender: ResponseSender, response: NativeTaskResponse) {
	if sender.send(response.encode()).is_err() {
		error!("Failed to send response");
	}
}

fn send_error(err_msg: String, sender: ResponseSender, error: NativeTaskError) {
	error!("{}", err_msg);
	send_response(sender, NativeTaskResponse::Err(error));
}

fn send_ok(sender: ResponseSender, ok_res: NativeTaskOk) {
	send_response(sender, NativeTaskResponse::Ok(ok_res));
}

async fn dispatch_as_signed<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
>(
	client: &mut RpcClient,
	signer: Arc<ParentchainTxSigner>,
	sender: AccountId,
	call: RuntimeCall,
	auth_type: Option<OmniAccountAuthType>,
) {
	let call = parentchain_api_interface::tx().omni_account().dispatch_as_signed(
		sender.to_subxt_type(),
		call,
		auth_type.map(|t| t.to_subxt_type()),
	);
	let tx = signer.sign(call).await;
	// notify parentchain - for now we continue even with error
	match client.submit_tx(&tx).await {
		Ok(_) => {
			debug!("Submitted dispatch_as_signed parentchain call")
		},
		Err(_) => {
			error!("Failed to submit dispatch_as_signed parentchain call",);
			signer.update_nonce().await
		},
	};
}

async fn notify_intent_accepted<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
>(
	client: &mut RpcClient,
	signer: Arc<ParentchainTxSigner>,
	account: AccountId,
	intent_id: IntentId,
	intent: Intent,
) {
	let call = parentchain_api_interface::tx().omni_account().intent_accepted(
		account.to_subxt_type(),
		intent_id,
		intent.to_subxt_type(),
	);

	let tx = signer.sign(call).await;

	// notify parentchain - for now we continue even with error
	match client.submit_tx(&tx).await {
		Ok(_) => {
			debug!("Submitted intent_accepted parentchain call for intent_id {}", intent_id)
		},
		Err(_) => {
			error!("Failed to submit intent_accepted parentchain call for intent_id {}", intent_id);
			signer.update_nonce().await
		},
	};
}

async fn notify_intent_completed<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
>(
	client: &mut RpcClient,
	signer: Arc<ParentchainTxSigner>,
	account: AccountId,
	intent_id: IntentId,
	detail: IntentCompletedDetail,
) {
	let call = parentchain_api_interface::tx().omni_account().intent_completed(
		account.to_subxt_type(),
		intent_id,
		detail,
	);

	let tx = signer.sign(call).await;

	// notify parentchain - for now we continue even with error
	match client.submit_tx(&tx).await {
		Ok(_) => {
			debug!("Submitted intent_completed parentchain call for intent_id {}", intent_id)
		},
		Err(_) => {
			error!(
				"Failed to submit intent_completed parentchain call for intent_id {}",
				intent_id
			);
			signer.update_nonce().await
		},
	};
}

async fn verify_google_code(
	pumpx_api: &dyn PumpxApi,
	access_token: &str,
	google_code: String,
	language: Option<String>,
) -> bool {
	debug!("Calling pumpx verify_google_code, code: {}", google_code);
	let verify_result = pumpx_api.verify_google_code(access_token, google_code, language).await;
	verify_result.map_or_else(
		|e| {
			error!("Google code verification request failed: {:?}", e);
			false
		},
		|res| {
			res.data.result.map_or_else(
				|| {
					error!("Google code verification response result is none");
					false
				},
				|success| success,
			)
		},
	)
}

/// Convert SerializablePackedUserOperation to aa_contracts_client::PackedUserOperation
fn convert_to_packed_user_op(
	user_op: SerializablePackedUserOperation,
) -> Result<aa_contracts_client::PackedUserOperation, String> {
	use std::str::FromStr;

	// Helper function to parse hex string to fixed bytes
	let parse_hex_fixed =
		|hex_str: &str, expected_len: usize, name: &str| -> Result<Vec<u8>, String> {
			let bytes = decode_hex(hex_str)
				.map_err(|e| format!("Invalid hex string '{}' '{}': {}", hex_str, name, e))?;
			if bytes.len() != expected_len {
				return Err(format!(
					"Expected {} bytes, got {} for '{}'",
					expected_len,
					bytes.len(),
					hex_str
				));
			}
			Ok(bytes)
		};

	Ok(aa_contracts_client::PackedUserOperation {
		sender: Address::from_str(&user_op.sender)
			.map_err(|e| format!("Invalid sender address '{}': {}", user_op.sender, e))?,
		nonce: U256::from(user_op.nonce),
		initCode: Bytes::from(
			decode_hex(&user_op.init_code).map_err(|e| format!("Invalid init_code hex: {}", e))?,
		),
		callData: Bytes::from(
			decode_hex(&user_op.call_data).map_err(|e| format!("Invalid call_data hex: {}", e))?,
		),
		accountGasLimits: {
			let bytes = parse_hex_fixed(&user_op.account_gas_limits, 32, "account_gas_limits")?;
			FixedBytes::from_slice(&bytes)
		},
		preVerificationGas: U256::from(user_op.pre_verification_gas),
		gasFees: {
			let bytes = parse_hex_fixed(&user_op.gas_fees, 32, "gas_fees")?;
			FixedBytes::from_slice(&bytes)
		},
		paymasterAndData: Bytes::from(
			decode_hex(&user_op.paymaster_and_data)
				.map_err(|e| format!("Invalid paymaster_and_data hex: {}", e))?,
		),
		signature: match user_op.signature {
			Some(sig) => {
				Bytes::from(decode_hex(&sig).map_err(|e| format!("Invalid signature hex: {}", e))?)
			},
			None => Bytes::new(), // Empty signature for unsigned operations
		},
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::{
		hex,
		primitives::{Bytes, U256},
	};
	use executor_core::types::SerializablePackedUserOperation;

	#[test]
	fn test_convert_to_packed_user_op() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0xdeadbeef".to_string(),
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let packed_user_op = convert_to_packed_user_op(serializable_user_op)
			.expect("Failed to convert SerializablePackedUserOperation");

		// Verify the conversion
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
		assert_eq!(packed_user_op.initCode, Bytes::from(hex::decode("deadbeef").unwrap()));
		assert_eq!(packed_user_op.callData, Bytes::from(hex::decode("cafebabe").unwrap()));
		assert_eq!(packed_user_op.preVerificationGas, U256::from(21000));
		assert_eq!(packed_user_op.paymasterAndData, Bytes::from(Vec::<u8>::new()));
		assert_eq!(packed_user_op.signature, Bytes::from(hex::decode("1234567890abcdef").unwrap()));
	}

	#[test]
	fn test_convert_to_packed_user_op_unsigned() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0xdeadbeef".to_string(),
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None, // Unsigned operation
		};

		let packed_user_op = convert_to_packed_user_op(serializable_user_op)
			.expect("Failed to convert unsigned SerializablePackedUserOperation");

		// Verify the signature is empty for unsigned operation
		assert!(packed_user_op.signature.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}

	#[test]
	fn test_convert_to_packed_user_op_empty_init_code() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "".to_string(), // Empty init_code
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let result = convert_to_packed_user_op(serializable_user_op);
		assert!(result.is_ok(), "Empty init_code should not cause an error: {:?}", result.err());

		let packed_user_op = result.unwrap();
		// Empty init_code should result in empty Bytes
		assert!(packed_user_op.initCode.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}

	#[test]
	fn test_convert_to_packed_user_op_0x_init_code() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(), // "0x" prefix only
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let result = convert_to_packed_user_op(serializable_user_op);
		assert!(result.is_ok(), "0x init_code should not cause an error: {:?}", result.err());

		let packed_user_op = result.unwrap();
		// "0x" init_code should result in empty Bytes
		assert!(packed_user_op.initCode.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}
}
