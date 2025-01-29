use crate::{
	authentication::{
		verify_auth_token_authentication, verify_email_authentication,
		verify_oauth2_authentication, verify_web3_authentication, Authentication,
	},
	server::RpcContext,
};
use executor_core::native_call::NativeCall;
use executor_primitives::{Nonce, ShardIdentifier};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct NativeCallAuthenticated {
	pub call: NativeCall,
	pub nonce: Nonce,
	pub authentication: Authentication,
}

pub fn verify_native_call_authenticated<
	AccountId,
	Header,
	RpcClient: SubstrateRpcClient<AccountId, Header>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient>,
>(
	ctx: Arc<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
	shard: &ShardIdentifier,
	handle: Handle,
	authenticated_call: &NativeCallAuthenticated,
) -> Result<(), crate::authentication::AuthenticationError> {
	let authentication_result = match authenticated_call.authentication {
		Authentication::Web3(ref signature) => verify_web3_authentication(
			signature,
			&authenticated_call.call,
			authenticated_call.nonce,
			&ctx.mrenclave,
			&shard,
		),
		Authentication::Email(ref verification_code) => verify_email_authentication(
			ctx,
			authenticated_call.call.sender_identity(),
			verification_code,
		),
		Authentication::OAuth2(ref oauth2_data) => verify_oauth2_authentication(
			ctx,
			handle,
			authenticated_call.call.sender_identity(),
			oauth2_data,
		),
		Authentication::AuthToken(ref auth_token) => verify_auth_token_authentication(
			ctx,
			handle,
			authenticated_call.call.sender_identity(),
			auth_token,
		),
	};
	authentication_result
}
