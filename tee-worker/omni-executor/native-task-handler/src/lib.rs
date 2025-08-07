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

// Gas estimation constants
/// Maximum verification gas limit to prevent DoS attacks
const MAX_VERIFICATION_GAS: u128 = 3_000_000;
/// Default verification gas for testing during binary search
const DEFAULT_VERIFICATION_GAS_FOR_TESTING: u64 = 1_000_000;
/// Minimum transaction gas as per EIP-155
const MIN_TRANSACTION_GAS: u64 = 21_000;
/// Maximum reasonable gas for normal operations
const MAX_NORMAL_GAS: u64 = 10_000_000;
/// Minimum gas for deployment operations
const MIN_DEPLOYMENT_GAS: u64 = 100_000;
/// Maximum gas for deployment operations
const MAX_DEPLOYMENT_GAS: u64 = 20_000_000;
/// Safety buffer percentage for verification gas
const VERIFICATION_GAS_BUFFER_PERCENT: u64 = 20;
/// Default paymaster verification gas limit
const DEFAULT_PAYMASTER_VERIFICATION_GAS: u128 = 100_000;
/// Default paymaster post-operation gas limit
const DEFAULT_PAYMASTER_POST_OP_GAS: u128 = 50_000;
/// Maximum allowed paymaster gas to prevent abuse
const MAX_PAYMASTER_GAS: u128 = 5_000_000;

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
		NativeTask::EstimateUserOpGas(
			omni_account,
			serializable_user_op,
			chain_id,
			wallet_index,
		) => {
			info!(
				"Processing EstimateUserOpGas for account {:?}, wallet_index: {}, chain_id: {}",
				omni_account, wallet_index, chain_id
			);

			// Get EntryPoint client for this chain
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

			// Convert SerializablePackedUserOperation to PackedUserOperation
			let packed_user_op = match convert_to_packed_user_op(serializable_user_op.clone()) {
				Ok(user_op) => user_op,
				Err(e) => {
					send_error(
						format!("Failed to convert UserOperation: {}", e),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				},
			};

			// Perform gas estimation (wallet_index can be used for wallet-specific optimizations)
			match estimate_user_op_gas(entry_point_client, packed_user_op, chain_id, wallet_index)
				.await
			{
				Ok(gas_estimates) => {
					info!("Gas estimation successful: {:?}", gas_estimates);
					if let Err(e) = response_sender.send(gas_estimates.encode()) {
						error!("Failed to send gas estimation response: {:?}", e);
					}
				},
				Err(e) => {
					send_error(
						format!("Gas estimation failed: {}", e),
						response_sender,
						NativeTaskError::InternalError,
					);
				},
			}
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
						hex::encode(user_op_hash_bytes),
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
						Ok(sig) => substrate_to_ethereum_signature(&sig).unwrap().to_vec(),
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

			// Run batch simulation for all UserOperations before submission
			info!("Running batch simulation for {} UserOperations", aa_user_ops.len());
			match entry_point_client.simulate_handle_ops(&aa_user_ops, beneficiary).await {
				Ok(simulation_results) => {
					for (index, result) in simulation_results.iter().enumerate() {
						info!(
							"UserOperation {} simulation successful. PreOpGas: {}, Paid: {}, AccountValidation: {}, PaymasterValidation: {}",
							index,
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
					info!(
						"All {} UserOperations passed batch simulation checks",
						aa_user_ops.len()
					);
				},
				Err(_) => {
					send_error(
						"Batch UserOperation simulation failed".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				},
			}

			// Submit all UserOperations via EntryPoint.handleOps() with retry logic
			let transaction_hash =
				match entry_point_client.handle_ops_with_retry(&aa_user_ops, beneficiary).await {
					Ok(tx_hash) => {
						// Return the actual transaction hash from handle_ops
						Some(tx_hash)
					},
					Err(_) => {
						send_error(
							"Failed to submit UserOperations to EntryPoint via handleOps after retries"
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

// Helper functions for gas limit packing/unpacking
fn pack_account_gas_limits(verification_gas: u128, call_gas: u128) -> FixedBytes<32> {
	let packed: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
	FixedBytes::from(packed.to_be_bytes())
}

#[cfg(test)]
fn unpack_verification_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = (value >> 128) & U256::from(u128::MAX);
	result.to::<u128>()
}

#[cfg(test)]
fn unpack_call_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = value & U256::from(u128::MAX);
	result.to::<u128>()
}

/// Convert Substrate signature to Ethereum ECDSA format
/// Returns signature in format: [r (32 bytes), s (32 bytes), v (1 byte)]
pub fn substrate_to_ethereum_signature(substrate_sig: &[u8]) -> Result<[u8; 65], &'static str> {
	if substrate_sig.len() != 65 {
		return Err("Invalid signature length");
	}

	// Parse as (r, s, v) format - most common
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
	r.copy_from_slice(&substrate_sig[0..32]);
	s.copy_from_slice(&substrate_sig[32..64]);
	let substrate_v = substrate_sig[64];

	// Convert recovery parameter: 0/1 -> 27/28
	let ethereum_v = match substrate_v {
		0 => 27,
		1 => 28,
		27 => 27, // Already Ethereum format
		28 => 28, // Already Ethereum format
		_ => return Err("Invalid recovery parameter"),
	};

	// Build Ethereum signature: [r, s, v]
	let mut ethereum_sig = [0u8; 65];
	ethereum_sig[0..32].copy_from_slice(&r);
	ethereum_sig[32..64].copy_from_slice(&s);
	ethereum_sig[64] = ethereum_v;

	Ok(ethereum_sig)
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

/// Estimate gas for a UserOperation using simulation
async fn estimate_user_op_gas(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
	wallet_index: u32,
) -> Result<NativeTaskOk, String> {
	info!(
		"Starting gas estimation for UserOperation on chain {} (wallet_index: {})",
		chain_id, wallet_index
	);

	// Step 1: Simulate validation to get base gas requirements
	let validation_result = entry_point_client
		.simulate_validation(user_op.clone())
		.await
		.map_err(|e| format!("Validation simulation failed: {:?}", e))?;

	// Extract preOpGas from validation result
	let pre_op_gas = validation_result.returnInfo.preOpGas;
	debug!("Validation simulation preOpGas: {}", pre_op_gas);

	// Step 2: Calculate verification gas limit based on validation result
	// Add buffer for safety
	let buffer_multiplier = U256::from(100 + VERIFICATION_GAS_BUFFER_PERCENT);
	let verification_gas_base = pre_op_gas.saturating_mul(buffer_multiplier) / U256::from(100);
	let verification_gas_limit = verification_gas_base.min(U256::from(MAX_VERIFICATION_GAS));

	// Step 3: Binary search for optimal call gas limit
	let call_gas_limit =
		estimate_call_gas_limit(entry_point_client.clone(), user_op.clone(), chain_id).await?;

	// Step 4: Calculate preVerificationGas (static + dynamic components)
	let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);
	let pre_verification_gas = static_pvg + dynamic_pvg;

	// Step 5: Extract paymaster gas limits if paymaster is present
	let (paymaster_verification_gas_limit, paymaster_post_op_gas_limit) =
		extract_paymaster_gas_limits(&user_op.paymasterAndData);

	// Convert to u128 for response, ensuring values are within bounds
	let call_gas_limit = call_gas_limit.try_into().map_err(|_| {
		format!("Call gas limit {} exceeds maximum supported value", call_gas_limit)
	})?;

	let verification_gas_limit = verification_gas_limit.try_into().map_err(|_| {
		format!("Verification gas limit {} exceeds maximum supported value", verification_gas_limit)
	})?;

	let pre_verification_gas = pre_verification_gas.try_into().map_err(|_| {
		format!("Pre-verification gas {} exceeds maximum supported value", pre_verification_gas)
	})?;

	let response = NativeTaskOk::EstimateUserOpGas {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
	};

	info!("Gas estimation complete: {:?}", response);
	Ok(response)
}

/// Binary search for optimal call gas limit following Rundler's approach
async fn estimate_call_gas_limit(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> Result<U256, String> {
	// Determine gas limits based on operation type
	let (min_gas, max_gas) = if !user_op.initCode.is_empty() {
		// Deployment operation requires higher gas limits
		(U256::from(MIN_DEPLOYMENT_GAS), U256::from(MAX_DEPLOYMENT_GAS))
	} else {
		// Normal operation
		(U256::from(MIN_TRANSACTION_GAS), U256::from(MAX_NORMAL_GAS))
	};

	// Step 1: Initial simulation at maximum to get baseline gas usage
	let mut test_user_op = user_op.clone();
	let verification_gas = U256::from(DEFAULT_VERIFICATION_GAS_FOR_TESTING);
	test_user_op.accountGasLimits =
		pack_account_gas_limits(verification_gas.to::<u128>(), max_gas.to::<u128>());

	let initial_result = entry_point_client
		.simulate_handle_ops(&vec![test_user_op], Address::ZERO)
		.await
		.map_err(|e| format!("Initial simulation failed: {:?}", e))?;

	if initial_result.is_empty() || !initial_result[0].targetSuccess {
		return Err("UserOperation validation failed at maximum gas limit".to_string());
	}

	// Extract actual gas used from the successful simulation
	let initial_gas_used = initial_result[0].paid;

	// Step 2: Set initial guess as 2x the gas used (accounts for 63/64ths rule)
	let initial_guess = initial_gas_used.saturating_mul(U256::from(2)).min(max_gas);

	info!("Initial gas simulation: used={}, initial_guess={}", initial_gas_used, initial_guess);

	// Step 3: Binary search with 10% tolerance (following Rundler's approach)
	let mut lower_bound = min_gas;
	let mut upper_bound = initial_guess;
	let mut iterations = 0;
	const MAX_ITERATIONS: u32 = 20;
	const TOLERANCE_PERCENT: u64 = 10; // Stop when bounds are within 10%

	while iterations < MAX_ITERATIONS {
		// Check if we've converged within tolerance
		if lower_bound > U256::ZERO {
			let range = upper_bound - lower_bound;
			let tolerance_threshold = lower_bound / U256::from(TOLERANCE_PERCENT);

			if range <= tolerance_threshold {
				debug!(
					"Binary search converged after {} iterations: range={}, threshold={}",
					iterations, range, tolerance_threshold
				);
				break;
			}
		}

		let mid_gas = (lower_bound + upper_bound) / U256::from(2);

		// Test if this gas limit works
		let mut test_user_op = user_op.clone();
		test_user_op.accountGasLimits =
			pack_account_gas_limits(verification_gas.to::<u128>(), mid_gas.to::<u128>());

		let test_result =
			entry_point_client.simulate_handle_ops(&vec![test_user_op], Address::ZERO).await;

		match test_result {
			Ok(results) if !results.is_empty() && results[0].targetSuccess => {
				// Simulation succeeded, try lower gas
				upper_bound = mid_gas;
				debug!("Binary search iteration {}: gas {} succeeded", iterations, mid_gas);
			},
			_ => {
				// Simulation failed, need more gas
				lower_bound = mid_gas + U256::from(1);
				debug!("Binary search iteration {}: gas {} failed", iterations, mid_gas);
			},
		}

		iterations += 1;
	}

	// Use the upper bound as our estimate (ensures success)
	let optimal_gas = upper_bound;

	// Add safety buffer based on chain
	let buffer_percent = match chain_id {
		1 => 50,     // Mainnet: 50% buffer
		42161 => 30, // Arbitrum: 30% buffer
		8453 => 30,  // Base: 30% buffer
		56 => 40,    // BSC: 40% buffer
		80084 => 20, // HyperEVM: 20% buffer
		_ => 40,     // Default: 40% buffer
	};

	let final_gas = optimal_gas.saturating_mul(U256::from(100 + buffer_percent)) / U256::from(100);

	info!("Call gas limit estimation: optimal={}, with_buffer={}", optimal_gas, final_gas);
	Ok(final_gas)
}

/// Calculate preVerificationGas split into static and dynamic components
fn calculate_pre_verification_gas(
	user_op: &aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> (U256, U256) {
	// EIP-2028 gas costs
	const GAS_PER_ZERO_BYTE: u64 = 4;
	const GAS_PER_NON_ZERO_BYTE: u64 = 16;
	const BASE_TRANSACTION_GAS: u64 = 21_000;
	const CREATE2_OVERHEAD_GAS: u64 = 32_000;
	const BUNDLE_OVERHEAD_GAS: u64 = 5_000; // Per-UserOp share of bundle transaction overhead

	// Helper function to calculate gas for bytes
	let calculate_bytes_gas = |data: &[u8]| -> U256 {
		let zero_bytes = data.iter().filter(|&&b| b == 0).count() as u64;
		let non_zero_bytes = (data.len() as u64) - zero_bytes;
		U256::from(zero_bytes * GAS_PER_ZERO_BYTE + non_zero_bytes * GAS_PER_NON_ZERO_BYTE)
	};

	// === STATIC PVG ===
	// These costs don't change based on network conditions
	let mut static_gas = U256::from(BASE_TRANSACTION_GAS + BUNDLE_OVERHEAD_GAS);

	// Calculate gas for UserOp calldata that will be included in the bundle
	static_gas += calculate_bytes_gas(&user_op.callData);
	static_gas += calculate_bytes_gas(&user_op.initCode);
	static_gas += calculate_bytes_gas(&user_op.paymasterAndData);
	static_gas += calculate_bytes_gas(&user_op.signature);

	// Add gas for fixed-size fields
	// sender (address as bytes20 padded to bytes32)
	static_gas += U256::from(20 * GAS_PER_NON_ZERO_BYTE + 12 * GAS_PER_ZERO_BYTE);
	// nonce (usually has many zero bytes)
	static_gas += U256::from(32 * GAS_PER_ZERO_BYTE);
	// accountGasLimits (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);
	// preVerificationGas (uint256)
	static_gas += U256::from(8 * GAS_PER_NON_ZERO_BYTE + 24 * GAS_PER_ZERO_BYTE);
	// gasFees (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);

	// Add deployment overhead if initCode is present
	if !user_op.initCode.is_empty() {
		static_gas += U256::from(CREATE2_OVERHEAD_GAS);
	}

	// === DYNAMIC PVG ===
	// L2-specific costs that can change based on L1 gas prices
	let dynamic_gas = calculate_l2_data_cost(user_op, chain_id);

	// Apply buffers
	let static_with_buffer = static_gas.saturating_mul(U256::from(110)) / U256::from(100); // 10% buffer
	let dynamic_with_buffer = if dynamic_gas > U256::ZERO {
		// L2s need higher buffer due to L1 gas price volatility
		dynamic_gas.saturating_mul(U256::from(125)) / U256::from(100) // 25% buffer for L2
	} else {
		U256::ZERO
	};

	debug!(
		"PreVerificationGas: static={}, dynamic={} (chain_id={}, total_bytes={})",
		static_with_buffer,
		dynamic_with_buffer,
		chain_id,
		user_op.callData.len() + user_op.initCode.len() + user_op.paymasterAndData.len()
	);

	(static_with_buffer, dynamic_with_buffer)
}

/// Calculate L2-specific data availability costs
fn calculate_l2_data_cost(
	user_op: &aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> U256 {
	// Check if this is an L2 network
	let is_l2 = matches!(
		chain_id,
		42161 | 421614 | // Arbitrum One, Arbitrum Sepolia
		10 | 11155420 |  // Optimism, Optimism Sepolia
		8453 | 84532 |   // Base, Base Sepolia
		137 | 80001 // Polygon, Mumbai
	);

	if !is_l2 {
		return U256::ZERO;
	}

	// Calculate total calldata size that needs to be posted to L1
	let total_bytes = user_op.callData.len()
		+ user_op.initCode.len()
		+ user_op.paymasterAndData.len()
		+ user_op.signature.len()
		+ 32 * 5; // Fixed fields

	// L2-specific multipliers (these would ideally come from an oracle)
	// These are rough estimates - production should use actual L1 gas price oracles
	let l1_data_cost_per_byte = match chain_id {
		42161 | 421614 => U256::from(140), // Arbitrum (uses Nitro compression)
		10 | 11155420 => U256::from(160),  // Optimism (uses bedrock compression)
		8453 | 84532 => U256::from(160),   // Base (same as Optimism)
		137 | 80001 => U256::from(50),     // Polygon (cheaper as sidechain)
		_ => U256::from(100),              // Default for unknown L2s
	};

	let dynamic_cost = U256::from(total_bytes) * l1_data_cost_per_byte;

	debug!(
		"L2 data cost calculation: chain_id={}, bytes={}, cost_per_byte={}, total={}",
		chain_id, total_bytes, l1_data_cost_per_byte, dynamic_cost
	);

	dynamic_cost
}

/// Extract and validate paymaster gas limits from paymasterAndData field
fn extract_paymaster_gas_limits(paymaster_and_data: &Bytes) -> (u128, u128) {
	// If no paymaster, return zeros
	if paymaster_and_data.len() < 20 {
		return (0, 0);
	}

	// paymasterAndData format:
	// [0:20] - paymaster address
	// [20:36] - paymaster verification gas limit (uint128)
	// [36:52] - paymaster post-op gas limit (uint128)
	// [52:] - paymaster data
	// check UserOperationLib.sol

	if paymaster_and_data.len() >= 52 {
		// Extract verification gas limit (bytes 20-36)
		let mut verification_bytes = [0u8; 16];
		verification_bytes.copy_from_slice(&paymaster_and_data[20..36]);
		let verification_gas_limit = u128::from_be_bytes(verification_bytes);

		// Extract post-op gas limit (bytes 36-52)
		let mut post_op_bytes = [0u8; 16];
		post_op_bytes.copy_from_slice(&paymaster_and_data[36..52]);
		let post_op_gas_limit = u128::from_be_bytes(post_op_bytes);

		// Validate gas limits to prevent abuse
		let validated_verification = if verification_gas_limit > MAX_PAYMASTER_GAS {
			debug!(
				"Paymaster verification gas {} exceeds maximum, using default",
				verification_gas_limit
			);
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else if verification_gas_limit == 0 {
			debug!("Paymaster verification gas is zero, using default");
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else {
			verification_gas_limit
		};

		let validated_post_op = if post_op_gas_limit > MAX_PAYMASTER_GAS {
			debug!("Paymaster post-op gas {} exceeds maximum, using default", post_op_gas_limit);
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else if post_op_gas_limit == 0 {
			debug!("Paymaster post-op gas is zero, using default");
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else {
			post_op_gas_limit
		};

		debug!(
			"Validated paymaster gas limits: verification={}, post_op={}",
			validated_verification, validated_post_op
		);

		(validated_verification, validated_post_op)
	} else {
		// Paymaster present but no gas limits specified, use defaults
		debug!("Paymaster present but gas limits not specified, using defaults");
		(DEFAULT_PAYMASTER_VERIFICATION_GAS, DEFAULT_PAYMASTER_POST_OP_GAS)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use aa_contracts_client::PackedUserOperation;
	use alloy::{
		hex,
		primitives::{Bytes, FixedBytes, U256},
	};
	use executor_core::types::SerializablePackedUserOperation;
	use executor_primitives::ChainId;

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

	#[test]
	fn test_pack_account_gas_limits() {
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;

		let packed = pack_account_gas_limits(verification_gas, call_gas);

		// Verify the packed format
		let expected: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		assert_eq!(packed.0, expected.to_be_bytes());
	}

	#[test]
	fn test_unpack_verification_gas_limit() {
		// Create a packed value with verification gas = 500000, call gas = 300000
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;
		let packed_value: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		let packed_bytes = FixedBytes::from(packed_value.to_be_bytes());

		let unpacked = unpack_verification_gas_limit(packed_bytes);
		assert_eq!(unpacked, verification_gas);
	}

	#[test]
	fn test_unpack_call_gas_limit() {
		// Create a packed value with verification gas = 500000, call gas = 300000
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;
		let packed_value: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		let packed_bytes = FixedBytes::from(packed_value.to_be_bytes());

		let unpacked = unpack_call_gas_limit(packed_bytes);
		assert_eq!(unpacked, call_gas);
	}

	#[test]
	fn test_pack_unpack_roundtrip() {
		let test_cases = vec![
			(0u128, 0u128),
			(1u128, 1u128),
			(u128::MAX, u128::MAX),
			(1_000_000u128, 500_000u128),
			(3_000_000u128, 10_000_000u128),
		];

		for (verification, call) in test_cases {
			let packed = pack_account_gas_limits(verification, call);
			let unpacked_verification = unpack_verification_gas_limit(packed);
			let unpacked_call = unpack_call_gas_limit(packed);

			assert_eq!(
				unpacked_verification,
				verification,
				"Verification gas mismatch for {:?}",
				(verification, call)
			);
			assert_eq!(unpacked_call, call, "Call gas mismatch for {:?}", (verification, call));
		}
	}

	#[test]
	fn test_calculate_pre_verification_gas_mainnet() {
		// Test with a simple UserOp for mainnet (no L2 costs)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00, 0x01, 0x02, 0x03]), // 4 bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]), // 65 bytes signature
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Static PVG should include base costs + calldata
		// Base: 21000 + 5000 = 26000
		// Calldata: sender(20) + nonce(~3) + initCode(0) + callData(4) + signature(65) + other fields
		// This is approximate since we need to calculate exact calldata costs
		assert!(static_pvg > U256::from(26_000), "Static PVG should be at least base costs");

		// Dynamic PVG should be 0 for mainnet
		assert_eq!(dynamic_pvg, U256::ZERO, "Dynamic PVG should be 0 for mainnet");
	}

	#[test]
	fn test_calculate_pre_verification_gas_arbitrum() {
		// Test with UserOp for Arbitrum (includes L2 data costs)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00; 100]), // 100 zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 42161; // Arbitrum One
		let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Static PVG should include base costs
		assert!(static_pvg > U256::from(26_000), "Static PVG should include base costs");

		// Dynamic PVG should be non-zero for Arbitrum (140 gas per byte)
		assert!(dynamic_pvg > U256::ZERO, "Dynamic PVG should be non-zero for Arbitrum");

		// Verify L2 multiplier is applied (140 gas per byte for Arbitrum)
		let total_bytes = user_op.sender.len() + 100 + 65; // Approximate total bytes
		let expected_min_dynamic = U256::from(total_bytes * 140);
		assert!(
			dynamic_pvg >= expected_min_dynamic,
			"Dynamic PVG should apply Arbitrum multiplier"
		);
	}

	#[test]
	fn test_calculate_pre_verification_gas_optimism() {
		// Test with UserOp for Optimism
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x01; 50]), // 50 non-zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 10; // Optimism
		let (_, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Dynamic PVG should use Optimism multiplier (160 gas per byte)
		assert!(dynamic_pvg > U256::ZERO, "Dynamic PVG should be non-zero for Optimism");

		// Should be higher than Arbitrum for same data
		let arbitrum_chain: ChainId = 42161; // Arbitrum One
		let (_, arbitrum_dynamic) = calculate_pre_verification_gas(&user_op, arbitrum_chain);
		assert!(
			dynamic_pvg > arbitrum_dynamic,
			"Optimism should have higher dynamic PVG than Arbitrum"
		);
	}

	#[test]
	fn test_calculate_pre_verification_gas_with_deployment() {
		// Test with initCode present (deployment scenario)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(0),                   // First transaction
			initCode: Bytes::from(vec![0x60; 200]), // 200 bytes of deployment code
			callData: Bytes::from(vec![]),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, _) = calculate_pre_verification_gas(&user_op, chain_id);

		// Should include CREATE2 overhead (32000 gas)
		// Base (26000) + CREATE2 (32000) + calldata costs
		assert!(static_pvg > U256::from(58_000), "Static PVG should include CREATE2 overhead");
	}

	#[test]
	fn test_calculate_pre_verification_gas_zero_bytes() {
		// Test calldata gas calculation with all zero bytes
		let user_op = PackedUserOperation {
			sender: "0x0000000000000000000000000000000000000000".parse().unwrap(),
			nonce: U256::from(0),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00; 1000]), // 1000 zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0x00; 65]), // All zero signature
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, _) = calculate_pre_verification_gas(&user_op, chain_id);

		// Zero bytes cost 4 gas each (EIP-2028)
		// Should be significantly lower than non-zero bytes
		let non_zero_op = PackedUserOperation {
			callData: Bytes::from(vec![0xff; 1000]), // 1000 non-zero bytes
			..user_op.clone()
		};
		let (non_zero_static, _) = calculate_pre_verification_gas(&non_zero_op, chain_id);

		assert!(
			static_pvg < non_zero_static,
			"Zero bytes should cost less gas than non-zero bytes"
		);
	}

	#[test]
	fn test_extract_paymaster_gas_limits_valid() {
		// Valid paymasterAndData with gas limits at bytes 20-52
		let mut paymaster_data = vec![0x11; 20]; // 20 bytes of paymaster address

		// Add verification gas limit (16 bytes, u128)
		let verification_gas = 150_000u128;
		paymaster_data.extend_from_slice(&verification_gas.to_be_bytes());

		// Add post-op gas limit (16 bytes, u128)
		let post_op_gas = 50_000u128;
		paymaster_data.extend_from_slice(&post_op_gas.to_be_bytes());

		// Add some extra data
		paymaster_data.extend_from_slice(&[0xff; 20]);

		let paymaster_and_data = Bytes::from(paymaster_data);
		let (extracted_verification, extracted_post_op) =
			extract_paymaster_gas_limits(&paymaster_and_data);

		assert_eq!(extracted_verification, verification_gas, "Verification gas should match");
		assert_eq!(extracted_post_op, post_op_gas, "Post-op gas should match");
	}

	#[test]
	fn test_extract_paymaster_gas_limits_short_data() {
		// Data shorter than 52 bytes
		let paymaster_data = vec![0x11; 30]; // Only 30 bytes
		let paymaster_and_data = Bytes::from(paymaster_data);

		let (verification, post_op) = extract_paymaster_gas_limits(&paymaster_and_data);

		// Should return defaults
		assert_eq!(verification, DEFAULT_PAYMASTER_VERIFICATION_GAS);
		assert_eq!(post_op, DEFAULT_PAYMASTER_POST_OP_GAS);
	}

	#[test]
	fn test_extract_paymaster_gas_limits_empty() {
		// Empty paymasterAndData
		let paymaster_and_data = Bytes::from(vec![]);

		let (verification, post_op) = extract_paymaster_gas_limits(&paymaster_and_data);

		// Should return zeros for no paymaster
		assert_eq!(verification, 0);
		assert_eq!(post_op, 0);
	}
}
