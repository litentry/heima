use crate::mailer_factory::MailerFactory;
use crate::oauth2_factory::OAuth2ConfigFactory;
use crate::{
	methods::register_methods,
	middlewares::{HttpMiddleware, RpcMiddleware},
	ShieldingKey,
};
use aa_contracts_client::EntryPointClient;
use binance_api::BinancePaymasterApi;
use config_loader::ConfigLoader;
use ethereum_rpc::AlloyRpcProvider;
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::aes256::Aes256Key;
use executor_storage::{StorageDB, WildmetaTimestampStorage};
use jsonrpsee::{server::Server, RpcModule};
use pumpx::PumpxApi;
use signer_client::SignerClient;
use std::collections::HashMap;
use std::marker::{Send, Sync};
use std::{env, net::SocketAddr, sync::Arc};
use tracing::info;
use wildmeta_api::WildmetaApi;

pub(crate) struct RpcContext<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
> {
	pub shielding_key: ShieldingKey,
	pub storage_db: Arc<StorageDB>,
	pub mailer_factory: Arc<MailerFactory>,
	pub oauth2_factory: Arc<OAuth2ConfigFactory>,
	pub jwt_rsa_private_key: Vec<u8>,
	pub pumpx_api: Arc<Box<dyn PumpxApi>>,
	// we could save copying client (and other objects) around when P-1527 is done
	// there could some a single `handler` that wraps up all accessible member variables
	pub signer_client: Arc<Box<dyn SignerClient>>,
	pub binance_api_client: Arc<dyn BinancePaymasterApi>,
	pub wildmeta_api: Arc<Box<dyn WildmetaApi>>,
	pub wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
	pub wildmeta_backend_ecdsa_pubkey: [u8; 33], // Compressed ECDSA public key for wildmeta backend signature verification
	pub bundler_private_key: [u8; 32],           // Bundler (accounting ECDSA) private key for export
	pub bundler_key_export_authorized_pubkey: [u8; 33], // Compressed ECDSA public key authorized to export bundler key
	pub ethereum_intent_executor: Arc<EthereumIntentExecutor>,
	pub solana_intent_executor: Arc<SolanaIntentExecutor>,
	pub cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	pub aes256_key: Aes256Key,
	pub entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
}

impl<
		EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
		SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
		CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	> RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: ShieldingKey,
		storage_db: Arc<StorageDB>,
		mailer_factory: Arc<MailerFactory>,
		oauth2_factory: Arc<OAuth2ConfigFactory>,
		jwt_rsa_private_key: Vec<u8>,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		signer_client: Arc<Box<dyn SignerClient>>,
		binance_api_client: Arc<dyn BinancePaymasterApi>,
		wildmeta_api: Arc<Box<dyn WildmetaApi>>,
		wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
		wildmeta_backend_ecdsa_pubkey: [u8; 33],
		bundler_private_key: [u8; 32],
		bundler_key_export_authorized_pubkey: [u8; 33],
		ethereum_intent_executor: Arc<EthereumIntentExecutor>,
		solana_intent_executor: Arc<SolanaIntentExecutor>,
		cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
		aes256_key: Aes256Key,
		entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
	) -> Self {
		Self {
			shielding_key,
			storage_db,
			mailer_factory,
			oauth2_factory,
			jwt_rsa_private_key,
			pumpx_api,
			signer_client,
			binance_api_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			wildmeta_backend_ecdsa_pubkey,
			bundler_private_key,
			bundler_key_export_authorized_pubkey,
			ethereum_intent_executor,
			solana_intent_executor,
			cross_chain_intent_executor,
			aes256_key,
			entry_point_clients,
		}
	}
}

#[allow(clippy::too_many_arguments)]
pub async fn start_server<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	port: u16,
	shielding_key: ShieldingKey,
	pumpx_api: Arc<Box<dyn PumpxApi>>,
	storage_db: Arc<StorageDB>,
	jwt_rsa_private_key: Vec<u8>,
	config_loader: &ConfigLoader,
	signer_client: Arc<Box<dyn SignerClient>>,
	binance_api_client: Arc<dyn BinancePaymasterApi>,
	wildmeta_api: Arc<Box<dyn WildmetaApi>>,
	wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
	wildmeta_backend_ecdsa_pubkey: [u8; 33],
	bundler_private_key: [u8; 32],
	bundler_key_export_authorized_pubkey: [u8; 33],
	ethereum_intent_executor: Arc<EthereumIntentExecutor>,
	solana_intent_executor: Arc<SolanaIntentExecutor>,
	cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	aes256_key: Aes256Key,
	entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
	let config_loader_arc = Arc::new(config_loader.clone());
	let mailer_factory = Arc::new(MailerFactory::new(config_loader_arc.clone()));
	let oauth2_factory = Arc::new(OAuth2ConfigFactory::new(config_loader_arc));

	let ctx = RpcContext::new(
		shielding_key,
		storage_db,
		mailer_factory,
		oauth2_factory,
		jwt_rsa_private_key.clone(),
		pumpx_api,
		signer_client,
		binance_api_client,
		wildmeta_api,
		wildmeta_timestamp_storage,
		wildmeta_backend_ecdsa_pubkey,
		bundler_private_key,
		bundler_key_export_authorized_pubkey,
		ethereum_intent_executor,
		solana_intent_executor,
		cross_chain_intent_executor,
		aes256_key,
		entry_point_clients,
	);
	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let address = format!("0.0.0.0:{}", port);
	let max_connections: u32 =
		env::var("OE_RPC_SERVER_MAX_CONNECTIONS").unwrap_or("100".to_string()).parse()?;
	let server = Server::builder()
		.max_connections(max_connections)
		.set_http_middleware(HttpMiddleware::create_builder())
		.set_rpc_middleware(RpcMiddleware::create_builder(jwt_rsa_private_key))
		.build(address.parse::<SocketAddr>()?)
		.await?;

	let handle = server.start(module);
	info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
