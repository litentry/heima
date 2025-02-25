use crate::{
	authentication::{
		verify_auth_token_authentication, verify_email_authentication,
		verify_oauth2_authentication, verify_web3_authentication, Authentication,
		AuthenticationError,
	},
	server::RpcContext,
};
use executor_core::native_operation::NativeOperation;
use executor_primitives::Nonce;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedOperation<OP: NativeOperation> {
	pub operation: OP,
	pub nonce: Nonce,
	pub authentication: Authentication,
}

pub fn verify_native_operation_authenticated<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	OP: NativeOperation,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
	authenticated_operation: &AuthenticatedOperation<OP>,
) -> Result<(), AuthenticationError> {
	let authentication_result = match authenticated_operation.authentication {
		Authentication::Web3(ref signature) => verify_web3_authentication(
			signature,
			&authenticated_operation.operation,
			authenticated_operation.nonce,
			ctx.mrenclave,
		),
		Authentication::Email(ref verification_code) => verify_email_authentication(
			ctx,
			authenticated_operation.operation.sender_identity(),
			verification_code,
		),
		Authentication::OAuth2(ref oauth2_data) => verify_oauth2_authentication(
			ctx,
			handle,
			authenticated_operation.operation.sender_identity(),
			oauth2_data,
		),
		Authentication::AuthToken(ref auth_token) => verify_auth_token_authentication(
			ctx,
			handle,
			authenticated_operation.operation.sender_identity(),
			auth_token,
		),
	};
	authentication_result
}
