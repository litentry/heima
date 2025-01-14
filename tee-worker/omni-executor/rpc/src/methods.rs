use jsonrpsee::RpcModule;

use crate::server::RpcContext;

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	// TODO: register methods here
	module
		.register_async_method("say_hello", |_params, _ctx, _| async { "hello" })
		.expect("Failed to register method");
}
