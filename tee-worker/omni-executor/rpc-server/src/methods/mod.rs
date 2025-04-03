use crate::server::RpcContext;
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

mod pumpx;
use pumpx::*;

mod omni;
use omni::*;

pub fn register_methods<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	register_omni(module);
	register_pumpx(module);
}
