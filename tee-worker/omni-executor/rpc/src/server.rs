use crate::methods::register_methods;
use jsonrpsee::{server::Server, RpcModule};
use std::net::SocketAddr;

// TODO: Define RpcContext
pub struct RpcContext {}

pub async fn start_server(port: &str, ctx: RpcContext) -> Result<(), Box<dyn std::error::Error>> {
	let address = format!("0.0.0.0:{}", port);
	let server = Server::builder().build(address.parse::<SocketAddr>()?).await?;

	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
