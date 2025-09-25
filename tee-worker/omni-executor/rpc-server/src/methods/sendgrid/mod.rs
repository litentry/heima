use crate::sendgrid_webhook::{process_sendgrid_webhook_batch, SendGridEvent};
use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct ProcessWebhookParams {
	pub events: Vec<SendGridEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessWebhookResponse {
	pub success: bool,
	pub processed_events: usize,
	pub message: String,
}

/// Register SendGrid webhook processing method
/// This is a temporary JSON-RPC method for testing webhook processing
/// In production, this would be handled by a dedicated HTTP endpoint
pub fn register_sendgrid_webhook_methods<
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
		.register_async_method("sendgrid_processWebhook", |params, _ctx, _| async move {
			let params = params.parse::<ProcessWebhookParams>().map_err(|e| {
				error!("[SENDGRID_WEBHOOK] Failed to parse webhook params: {:?}", e);
				ErrorObject::owned(
					-32602,
					"Invalid parameters",
					Some(format!("Failed to parse webhook parameters: {}", e)),
				)
			})?;

			let event_count = params.events.len();
			process_sendgrid_webhook_batch(params.events);

			let response = ProcessWebhookResponse {
				success: true,
				processed_events: event_count,
				message: format!("Processed {} webhook events", event_count),
			};

			Ok::<ProcessWebhookResponse, ErrorObject>(response)
		})
		.expect("Failed to register sendgrid_processWebhook method");
}
