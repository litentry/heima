use crate::server::RpcContext;
use heima_authentication::web3::{generate_message_code, MESSAGE_CODE_PERIOD};
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct MessageCodeResponse {
	pub message_code: String,
	pub seconds_left: u64,
}

pub fn register_get_message_code(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getMessageCode", |_params, _ctx, _| async move {
			let (message_code, seconds_left) = generate_message_code(MESSAGE_CODE_PERIOD);
			let response = MessageCodeResponse { message_code, seconds_left };
			Ok::<MessageCodeResponse, ErrorObject>(response)
		})
		.expect("Failed to register omni_getMessageCode method");
}
