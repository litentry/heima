mod aes256_key_store;
mod native_call_handlers;
mod native_query_handlers;
mod types;
pub use aes256_key_store::Aes256KeyStore;

use executor_core::{
	intent_executor::IntentExecutor,
	native_operation::{NativeCall, NativeQuery},
};
use executor_crypto::aes256::Aes256Key;
use executor_primitives::OmniAccountAuthType;
use executor_storage::StorageDB;
use native_call_handlers::handle_native_call;
use native_query_handlers::handle_native_query;
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient, SubxtClientFactory,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use types::{NativeOperationError, NativeOperationOk};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;

pub type NativeTaskSender = mpsc::Sender<NativeTask>;

type NativeOperationResponse = Result<NativeOperationOk, NativeOperationError>;

pub type ParentchainTxSigner = TransactionSigner<
	SubstrateKeyStore,
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

pub enum NativeTaskOperation {
	Call(NativeCall),
	Query(NativeQuery),
}

pub struct NativeTask {
	pub operation: NativeTaskOperation,
	pub auth_type: OmniAccountAuthType,
	pub response_sender: ResponseSender,
}

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
			phantom_header: PhantomData,
			phantom_rpc_client: PhantomData,
		}
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
	let (sender, mut receiver) = mpsc::channel::<NativeTask>(buffer);

	tokio::spawn(async move {
		while let Some(task) = receiver.recv().await {
			handle_native_task(ctx.clone(), task).await;
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
	task: NativeTask,
) {
	match task.operation {
		NativeTaskOperation::Call(native_call) => {
			handle_native_call(ctx.clone(), native_call, task.auth_type, task.response_sender)
				.await;
		},
		NativeTaskOperation::Query(native_query) => {
			handle_native_query(ctx.clone(), native_query, task.response_sender).await;
		},
	};
}
