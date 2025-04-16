use crate::server::RpcContext;
use heima_authentication::web3::{generate_message_code, HeimaMessagePayload, MESSAGE_CODE_PERIOD};
use jsonrpsee::{types::ErrorObject, RpcModule};

pub fn register_get_message_code(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getMessageCode", |_params, _ctx, _| async move {
			let message_payload =
				HeimaMessagePayload { message_code: generate_message_code(MESSAGE_CODE_PERIOD) };
			Ok::<HeimaMessagePayload, ErrorObject>(message_payload)
		})
		.expect("Failed to register omni_getMessageCode method");
}
