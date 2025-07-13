use crate::{
	methods::register_methods,
	middlewares::{HttpMiddleware, RpcMiddleware},
	wildmeta_api::WildmetaApi,
	ShieldingKey,
};
use config_loader::ConfigLoader;
use ethereum_rpc::AlloyRpcProvider;
use executor_storage::{StorageDB, WildmetaTimestampStorage};
use heima_identity_verification::web2::email::Mailer;
use jsonrpsee::{server::Server, RpcModule};
use native_task_handler::NativeTaskSender;
use pumpx::PumpxApi;
use signer_client::SignerClient;
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
use tracing::info;

pub(crate) struct RpcContext {
	pub shielding_key: ShieldingKey,
	pub native_task_sender: Arc<NativeTaskSender>,
	pub storage_db: Arc<StorageDB>,
	pub mailer: Mailer,
	pub jwt_rsa_private_key: Vec<u8>,
	pub google_client_id: String,
	pub google_client_secret: String,
	pub pumpx_api: Arc<Box<dyn PumpxApi>>,
	pub rpc_clients: Arc<HashMap<u64, Arc<AlloyRpcProvider>>>,
	// we could save copying client (and other objects) around when P-1527 is done
	// there could some a single `handler` that wraps up all accessible member variables
	pub signer_client: Arc<Box<dyn SignerClient>>,
	pub wildmeta_api: Arc<Box<dyn WildmetaApi>>,
	pub wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
}

impl RpcContext {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: ShieldingKey,
		native_task_sender: Arc<NativeTaskSender>,
		storage_db: Arc<StorageDB>,
		mailer: Mailer,
		jwt_rsa_private_key: Vec<u8>,
		google_client_id: String,
		google_client_secret: String,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		rpc_clients: Arc<HashMap<u64, Arc<AlloyRpcProvider>>>,
		signer_client: Arc<Box<dyn SignerClient>>,
		wildmeta_api: Arc<Box<dyn WildmetaApi>>,
		wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
	) -> Self {
		Self {
			shielding_key,
			native_task_sender,
			storage_db,
			mailer,
			jwt_rsa_private_key,
			google_client_id,
			google_client_secret,
			pumpx_api,
			rpc_clients,
			signer_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
		}
	}
}

#[allow(clippy::too_many_arguments)]
pub async fn start_server(
	port: u16,
	shielding_key: ShieldingKey,
	native_task_sender: Arc<NativeTaskSender>,
	pumpx_api: Arc<Box<dyn PumpxApi>>,
	storage_db: Arc<StorageDB>,
	jwt_rsa_private_key: Vec<u8>,
	config_loader: &ConfigLoader,
	rpc_clients: Arc<HashMap<u64, Arc<AlloyRpcProvider>>>,
	signer_client: Arc<Box<dyn SignerClient>>,
	wildmeta_api: Arc<Box<dyn WildmetaApi>>,
	wildmeta_timestamp_storage: Arc<WildmetaTimestampStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
	let mailer = Mailer::new(
		config_loader.mailer_api_host.clone(),
		config_loader.mailer_api_key.clone(),
		config_loader.mailer_from_email.clone(),
		config_loader.mailer_from_name.clone(),
	);

	let ctx = RpcContext::new(
		shielding_key,
		native_task_sender,
		storage_db,
		mailer,
		jwt_rsa_private_key.clone(),
		config_loader.google_client_id.clone(),
		config_loader.google_client_secret.clone(),
		pumpx_api,
		rpc_clients,
		signer_client,
		wildmeta_api,
		wildmeta_timestamp_storage,
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
