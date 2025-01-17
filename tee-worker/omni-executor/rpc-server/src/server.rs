use crate::{methods::register_methods, ShieldingKey};
use jsonrpsee::{server::Server, RpcModule};
use native_call_executor::NativeCallSender;
use std::net::SocketAddr;
use std::sync::Arc;

pub(crate) struct RpcContext {
	pub shielding_key: ShieldingKey,
	pub native_call_sender: Arc<NativeCallSender>,
	pub mrenclave: [u8; 32],
}

pub async fn start_server(
	port: &str,
	shielding_key: ShieldingKey,
	native_call_sender: Arc<NativeCallSender>,
	mrenclave: [u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
	let address = format!("0.0.0.0:{}", port);
	let server = Server::builder().build(address.parse::<SocketAddr>()?).await?;

	let ctx = RpcContext { shielding_key, native_call_sender, mrenclave };
	let mut module = RpcModule::new(ctx);
	register_methods(&mut module);

	let handle = server.start(module);
	log::info!("Server listening on port {}", port);
	tokio::spawn(handle.stopped());

	Ok(())
}
