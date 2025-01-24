use crate::{methods::register_methods, ShieldingKey};
use heima_identity_verification::web2::email::Mailer;
use jsonrpsee::{server::Server, RpcModule};
use native_task_handler::NativeTaskSender;
use parentchain_rpc_client::{CustomConfig, SubxtClient};
use std::{env, net::SocketAddr, sync::Arc};
use storage::StorageDB;

pub(crate) struct RpcContext {
	pub shielding_key: ShieldingKey,
	pub native_task_sender: Arc<NativeTaskSender>,
	pub parentchain_rpc_client: Arc<SubxtClient<CustomConfig>>,
	pub storage_db: Arc<StorageDB>,
	pub mrenclave: [u8; 32],
	pub mailer: Mailer,
	pub jwt_secret: String,
}

pub async fn start_server(
	port: &str,
	parentchain_rpc_client: Arc<SubxtClient<CustomConfig>>,
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

	let ctx = RpcContext {
		shielding_key,
		native_task_sender,
		parentchain_rpc_client,
		mrenclave,
		storage_db,
		mailer,
		jwt_secret,
	};
	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
