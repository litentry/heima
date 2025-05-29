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

#[derive(Debug, Clone)]
pub struct HttpExtensions {
	pub authorization_header: String,
}

pub struct HttpMiddleware;

impl HttpMiddleware {
	pub fn create_builder() -> HttpServiceBuilder {
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
			req.extensions_mut().insert(HttpExtensions { authorization_header: value });
		}

		self.inner.call(req)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use http::{HeaderValue, Method};
	use jsonrpsee::server::HttpBody;
	use std::convert::Infallible;
	use std::future::{ready, Ready};
	use tower::{Service, ServiceExt};

	// Mock service for testing
	#[derive(Clone)]
	struct MockService;

	impl Service<Request<HttpBody>> for MockService {
		type Response = String;
		type Error = Infallible;
		type Future = Ready<Result<Self::Response, Self::Error>>;

		fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
			Poll::Ready(Ok(()))
		}

		fn call(&mut self, req: Request<HttpBody>) -> Self::Future {
			// Extract the authorization header from request extensions
			let http_extensions = req.extensions().get::<HttpExtensions>().cloned();
			let response = match http_extensions {
				Some(ext) => format!("Auth: {}", ext.authorization_header),
				None => "No auth".to_string(),
			};
			ready(Ok(response))
		}
	}

	#[tokio::test]
	async fn test_authorization_header_extractor_with_valid_header() {
		let layer = AuthorizationHeaderExtractorLayer;
		let mut service = layer.layer(MockService);

		let req = Request::builder()
			.method(Method::POST)
			.uri("http://example.com")
			.header(AUTHORIZATION, "Bearer token123")
			.body(HttpBody::empty())
			.unwrap();

		let response = service.ready().await.unwrap().call(req).await.unwrap();
		assert_eq!(response, "Auth: Bearer token123");
	}

	#[tokio::test]
	async fn test_authorization_header_extractor_without_header() {
		let layer = AuthorizationHeaderExtractorLayer;
		let mut service = layer.layer(MockService);

		let req = Request::builder()
			.method(Method::POST)
			.uri("http://example.com")
			.body(HttpBody::empty())
			.unwrap();

		let response = service.ready().await.unwrap().call(req).await.unwrap();
		assert_eq!(response, "No auth");
	}

	#[tokio::test]
	async fn test_authorization_header_extractor_with_invalid_utf8() {
		let layer = AuthorizationHeaderExtractorLayer;
		let mut service = layer.layer(MockService);

		let mut req = Request::builder()
			.method(Method::POST)
			.uri("http://example.com")
			.body(HttpBody::empty())
			.unwrap();

		// Insert invalid UTF-8 header value
		req.headers_mut()
			.insert(AUTHORIZATION, HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap());

		let response = service.ready().await.unwrap().call(req).await.unwrap();
		assert_eq!(response, "No auth");
	}

	#[tokio::test]
	async fn test_authorization_header_extractor_with_empty_header() {
		let layer = AuthorizationHeaderExtractorLayer;
		let mut service = layer.layer(MockService);

		let req = Request::builder()
			.method(Method::POST)
			.uri("http://example.com")
			.header(AUTHORIZATION, "")
			.body(HttpBody::empty())
			.unwrap();

		let response = service.ready().await.unwrap().call(req).await.unwrap();
		assert_eq!(response, "Auth: ");
	}
}
