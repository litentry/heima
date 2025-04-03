use crate::server::RpcContext;
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

pub fn register_pumpx<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	register_request_jwt(module);
	register_export_wallet(module);
}
