use crate::server::RpcContext;
use executor_primitives::{utils::hex::ToHexPrefixed, Identity, Web2IdentityType};
use executor_storage::{OAuth2StateVerifierStorage, Storage};
use heima_identity_verification::web2::google;
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

pub fn register_get_oauth2_google_authorization_url<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method(
			"omni_getOAuth2GoogleAuthorizationUrl",
			|params, ctx, _| async move {
				match params.parse::<(String, String)>() {
					Ok((google_account, redirect_uri)) => {
						let google_identity =
							Identity::from_web2_account(&google_account, Web2IdentityType::Google);
						let authorization_data =
							google::get_authorize_data(&ctx.google_client_id, &redirect_uri);
						let storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
						storage
							.insert(google_identity.hash(), authorization_data.state.clone())
							.map_err(|_| ErrorCode::InternalError)?;
						Ok::<String, ErrorObject>(authorization_data.authorize_url.to_hex())
					},
					Err(_) => Err(ErrorCode::ParseError.into()),
				}
			},
		)
		.expect("Failed to register omni_getOAuth2GoogleAuthorizationUrl method");
}
