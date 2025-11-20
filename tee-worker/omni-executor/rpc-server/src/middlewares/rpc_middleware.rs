use crate::{
	error_code::AUTH_VERIFICATION_FAILED_CODE, methods::PROTECTED_METHODS,
	middlewares::HttpExtensions, verify_auth::verify_auth_token_authentication,
};
use jsonrpsee::{
	server::middleware::rpc::{Batch, Notification, RpcServiceBuilder, RpcServiceT},
	types::{ErrorObject, Request},
	MethodResponse,
};
use oe_core::auth::constants::{AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_ID_TYPE};
use tower::layer::util::Stack;

#[derive(Clone, Debug)]
pub struct RpcExtensions {
	pub sender: String,
	#[allow(dead_code)]
	pub client_id: String,
}

pub struct RpcMiddleware;

impl RpcMiddleware {
	pub fn create_builder(
		rsa_private_key: Vec<u8>,
	) -> RpcServiceBuilder<Stack<AuthRpcLayer, tower::layer::util::Identity>> {
		RpcServiceBuilder::new().layer(AuthRpcLayer { rsa_private_key })
	}
}

#[derive(Clone)]
pub struct AuthRpcLayer {
	rsa_private_key: Vec<u8>,
}

impl<S> tower::Layer<S> for AuthRpcLayer {
	type Service = AuthRpcService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		AuthRpcService { service: inner, rsa_private_key: self.rsa_private_key.clone() }
	}
}

#[derive(Clone)]
pub struct AuthRpcService<S> {
	service: S,
	rsa_private_key: Vec<u8>,
}

impl<S> RpcServiceT for AuthRpcService<S>
where
	S: RpcServiceT<
			MethodResponse = MethodResponse,
			NotificationResponse = MethodResponse,
			BatchResponse = MethodResponse,
		> + Send
		+ Sync
		+ Clone
		+ 'static,
{
	type MethodResponse = S::MethodResponse;
	type NotificationResponse = S::NotificationResponse;
	type BatchResponse = S::BatchResponse;

	fn call<'a>(
		&self,
		mut req: Request<'a>,
	) -> impl std::future::Future<Output = Self::MethodResponse> + Send + 'a {
		let service = self.service.clone();
		let rsa_private_key = self.rsa_private_key.clone();

		async move {
			if PROTECTED_METHODS.contains(&req.method_name()) {
				if let Some(http_extensions) = req.extensions().get::<HttpExtensions>() {
					let token =
						http_extensions.authorization_header.trim_start_matches("Bearer ").trim();
					match verify_auth_token_authentication(
						&rsa_private_key,
						token,
						auth_token_type_for_method(req.method_name()),
						false,
					) {
						Ok(claims) => {
							req.extensions_mut().insert(RpcExtensions {
								sender: claims.sub.clone(),
								client_id: claims.aud.clone(),
							});
						},
						Err(e) => {
							tracing::debug!("Authentication failed: {}", e);
							return MethodResponse::error(
								req.id,
								ErrorObject::borrowed(
									AUTH_VERIFICATION_FAILED_CODE,
									"Authentication failed",
									None,
								),
							);
						},
					}
				} else {
					tracing::error!("No authentication header found");
					return MethodResponse::error(
						req.id,
						ErrorObject::borrowed(-32603, "Internal error", None),
					);
				}
			}

			service.call(req).await
		}
	}

	fn batch<'a>(
		&self,
		batch: Batch<'a>,
	) -> impl std::future::Future<Output = Self::BatchResponse> + Send + 'a {
		self.service.batch(batch)
	}

	fn notification<'a>(
		&self,
		notification: Notification<'a>,
	) -> impl std::future::Future<Output = Self::NotificationResponse> + Send + 'a {
		self.service.notification(notification)
	}
}

// Defines the methods that require "access" auth token type
const ACCESS_TOKEN_PROTECTED_METHODS: [&str; 2] =
	["omni_notifyLimitOrderResult", "omni_signLimitOrder"];

fn auth_token_type_for_method(method: &str) -> &'static str {
	if ACCESS_TOKEN_PROTECTED_METHODS.contains(&method) {
		AUTH_TOKEN_ACCESS_TYPE
	} else {
		AUTH_TOKEN_ID_TYPE
	}
}
