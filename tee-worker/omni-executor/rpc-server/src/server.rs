use crate::{methods::register_methods, ShieldingKey};
use executor_storage::StorageDB;
use heima_identity_verification::web2::email::Mailer;
use jsonrpsee::{server::Server, RpcModule};
use native_task_handler::NativeTaskSender;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::{env, marker::PhantomData, net::SocketAddr, sync::Arc};

pub(crate) struct RpcContext<
	AccountId,
	Header,
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
> {
	pub shielding_key: ShieldingKey,
	pub native_task_sender: Arc<NativeTaskSender>,
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub mrenclave: [u8; 32],
	pub mailer: Mailer,
	pub jwt_secret: String,
	pub google_client_id: String,
	pub google_client_secret: String,
	phantom_account_id: PhantomData<AccountId>,
	phantom_header: PhantomData<Header>,
	phantom_hash: PhantomData<Hash>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		AccountId,
		Header,
		Hash,
		RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
		RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
	> RpcContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: ShieldingKey,
		native_task_sender: Arc<NativeTaskSender>,
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		storage_db: Arc<StorageDB>,
		mrenclave: [u8; 32],
		mailer: Mailer,
		jwt_secret: String,
		google_client_id: String,
		google_client_secret: String,
	) -> Self {
		Self {
			shielding_key,
			native_task_sender,
			parentchain_rpc_client_factory,
			storage_db,
			mrenclave,
			mailer,
			jwt_secret,
			google_client_id,
			google_client_secret,
			phantom_account_id: PhantomData,
			phantom_header: PhantomData,
			phantom_hash: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}
}

pub async fn start_server<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	Hash: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient> + Send + Sync + 'static,
>(
	port: &str,
	parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	shielding_key: ShieldingKey,
	native_task_sender: Arc<NativeTaskSender>,
	storage_db: Arc<StorageDB>,
	mrenclave: [u8; 32],
	jwt_secret: String,
) -> Result<(), Box<dyn std::error::Error>> {
	let address = format!("0.0.0.0:{}", port);
	let server = Server::builder().build(address.parse::<SocketAddr>()?).await?;

	// TODO: move to config
	let mailer_api_key = env::var("SENDGRID_API_KEY").unwrap_or("".to_string());
	let mailer_from_email = env::var("SENDGRID_FROM_EMAIL").unwrap_or("".to_string());
	let mailer_from_name = env::var("SENDGRID_FROM_NAME").unwrap_or("".to_string());
	let mailer = Mailer::new(mailer_api_key, mailer_from_email, mailer_from_name);

	let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or("".to_string());
	let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap_or("".to_string());

	let ctx = RpcContext::new(
		shielding_key,
		native_task_sender,
		parentchain_rpc_client_factory,
		storage_db,
		mrenclave,
		mailer,
		jwt_secret,
		google_client_id,
		google_client_secret,
	);
	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
