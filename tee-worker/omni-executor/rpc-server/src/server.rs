use crate::{methods::register_methods, ShieldingKey};
use jsonrpsee::{server::Server, RpcModule};
use std::net::SocketAddr;
use std::sync::Arc;

pub(crate) struct RpcContext {
	pub shielding_key: Arc<ShieldingKey>,
}

pub async fn start_server(
	port: &str,
	shielding_key: Arc<ShieldingKey>,
) -> Result<(), Box<dyn std::error::Error>> {
	let address = format!("0.0.0.0:{}", port);
	let server = Server::builder().build(address.parse::<SocketAddr>()?).await?;
	let ctx = RpcContext { shielding_key };

	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
