use crate::{server::RpcContext, Deserialize};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{Hashable, Identity, Web2IdentityType};
use executor_storage::{Storage, VerificationCodeStorage};
use heima_identity_verification::web2::email::{
	generate_verification_code, send_verification_email,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct RequestEmailVerificationCodeParams {
	pub client_id: String,
	pub user_email: String,
}

pub fn register_request_email_verification_code<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module
		.register_async_method("omni_requestEmailVerificationCode", |params, ctx, _| async move {
			let params = params.parse::<RequestEmailVerificationCodeParams>()?;

			debug!(
				"Received omni_requestEmailVerificationCode, client_id: {}, user_email: {}",
				params.client_id, params.user_email
			);

			let email_identity =
				Identity::from_web2_account(&params.user_email, Web2IdentityType::Email);
			let omni_account = email_identity.to_omni_account(&params.client_id);
			let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
			let verification_code = generate_verification_code();

			verification_code_storage
				.insert(&omni_account.hash(), verification_code.clone())
				.map_err(|_| ErrorCode::InternalError)?;

			send_verification_email(&*ctx.mailer, params.user_email, verification_code)
				.await
				.map_err(|_| {
					error!("Failed to send verification email");
					ErrorCode::InternalError
				})?;

			Ok::<(), ErrorObject>(())
		})
		.expect("Failed to register omni_requestEmailVerificationCode method");
}
