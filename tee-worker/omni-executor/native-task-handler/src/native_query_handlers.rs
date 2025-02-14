use crate::{ResponseSender, TaskHandlerContext};
use executor_core::native_operation::NativeQuery;
use executor_primitives::OmniAccountAuthType;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::sync::Arc;

pub async fn handle_native_query<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	_ctx: Arc<TaskHandlerContext<Header, RpcClient, RpcClientFactory>>,
	_query: NativeQuery,
	_auth_type: OmniAccountAuthType,
	_response_sender: ResponseSender,
) {
	todo!()
}
