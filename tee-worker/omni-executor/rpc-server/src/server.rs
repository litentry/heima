use crate::{methods::register_methods, ShieldingKey};
use executor_primitives::MrEnclave;
use executor_storage::StorageDB;
use heima_identity_verification::web2::email::Mailer;
use intent_core::IntentIdStore;
use jsonrpsee::{server::Server, RpcModule};
use native_task_handler::NativeTaskSender;
use std::{env, net::SocketAddr, sync::Arc};

pub(crate) struct RpcContext {
	pub shielding_key: ShieldingKey,
	pub native_task_sender: Arc<NativeTaskSender>,
	pub storage_db: Arc<StorageDB>,
	pub mrenclave: MrEnclave,
	pub mailer: Mailer,
	pub jwt_rsa_private_key: Vec<u8>,
	pub google_client_id: String,
	pub google_client_secret: String,
	pub intent_id_store: Arc<Box<dyn IntentIdStore>>,
}

impl RpcContext {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: ShieldingKey,
		native_task_sender: Arc<NativeTaskSender>,
		storage_db: Arc<StorageDB>,
		mrenclave: [u8; 32],
		mailer: Mailer,
		jwt_rsa_private_key: Vec<u8>,
		google_client_id: String,
		google_client_secret: String,
		intent_id_store: Arc<Box<dyn IntentIdStore>>,
	) -> Self {
		Self {
			shielding_key,
			native_task_sender,
			storage_db,
			mrenclave: MrEnclave::from(mrenclave),
			mailer,
			jwt_rsa_private_key,
			google_client_id,
			google_client_secret,
			intent_id_store,
		}
	}
}

pub async fn start_server(
	port: u16,
	shielding_key: ShieldingKey,
	native_task_sender: Arc<NativeTaskSender>,
	storage_db: Arc<StorageDB>,
	mrenclave: [u8; 32],
	jwt_rsa_private_key: Vec<u8>,
	intent_id_store: Arc<Box<dyn IntentIdStore>>,
) -> Result<(), Box<dyn std::error::Error>> {
	let address = format!("0.0.0.0:{}", port);
	let max_connections: u32 =
		env::var("OE_RPC_SERVER_MAX_CONNECTIONS").unwrap_or("100".to_string()).parse()?;
	let server = Server::builder()
		.max_connections(max_connections)
		.build(address.parse::<SocketAddr>()?)
		.await?;

	// TODO: move to config
	let mailer_api_key = env::var("OE_SENDGRID_API_KEY").unwrap_or("".to_string());
	let mailer_from_email = env::var("OE_SENDGRID_FROM_EMAIL").unwrap_or("".to_string());
	let mailer_from_name = env::var("OE_SENDGRID_FROM_NAME").unwrap_or("".to_string());
	let mailer = Mailer::new(mailer_api_key, mailer_from_email, mailer_from_name);

	let google_client_id = env::var("OE_GOOGLE_CLIENT_ID").unwrap_or("".to_string());
	let google_client_secret = env::var("OE_GOOGLE_CLIENT_SECRET").unwrap_or("".to_string());

	let ctx = RpcContext::new(
		shielding_key,
		native_task_sender,
		storage_db,
		mrenclave,
		mailer,
		jwt_rsa_private_key,
		google_client_id,
		google_client_secret,
		intent_id_store,
	);
	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
