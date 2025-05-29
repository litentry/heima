use http::{header::AUTHORIZATION, Request};
use jsonrpsee::server::HttpBody;
use std::iter::once;
use std::task::{Context, Poll};
use tower::{
	layer::util::{Identity, Stack},
	ServiceBuilder,
};
use tower::{Layer, Service};
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;

type HttpServiceBuilder = ServiceBuilder<
	Stack<AuthorizationHeaderExtractorLayer, Stack<SetSensitiveRequestHeadersLayer, Identity>>,
>;

pub struct HttpMiddleware;

impl HttpMiddleware {
	pub fn new() -> HttpServiceBuilder {
		tower::ServiceBuilder::new()
			.layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
			.layer(AuthorizationHeaderExtractorLayer)
	}
}

#[derive(Clone)]
pub struct AuthorizationHeaderExtractorLayer;

impl<S> Layer<S> for AuthorizationHeaderExtractorLayer {
	type Service = AuthorizationHeaderExtractorService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		AuthorizationHeaderExtractorService { inner }
	}
}

#[derive(Clone)]
pub struct AuthorizationHeaderExtractorService<S> {
	inner: S,
}

impl<S> Service<Request<HttpBody>> for AuthorizationHeaderExtractorService<S>
where
	S: Service<Request<HttpBody>>,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, mut req: Request<HttpBody>) -> Self::Future {
		let header_value = req
			.headers()
			.get(AUTHORIZATION)
			.and_then(|hv| hv.to_str().ok())
			.map(|value_str| value_str.to_string());

		if let Some(value) = header_value {
			req.extensions_mut().insert(value);
		}

		self.inner.call(req)
	}
}
