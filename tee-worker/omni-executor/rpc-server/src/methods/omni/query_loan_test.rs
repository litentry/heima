use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::AccountId;
use executor_storage::LoanRecord;
use jsonrpsee::RpcModule;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct QueryLoanTestParams {
	pub omni_account: String,
	pub nonce: Option<u64>,
}

#[derive(Clone, Serialize)]
pub struct QueryLoanTestResponse {
	pub records: HashMap<String, LoanRecord>,
}

pub fn register_query_loan_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_method("omni_queryLoanTest", |params, ctx, _ext| {
			let params = params.parse::<QueryLoanTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_queryLoanTest, params: {:?}", params);

			let address_bytes =
				hex::decode(params.omni_account.strip_prefix("0x").unwrap_or(&params.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason("Failed to decode omni account hex string"),
						)
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
						"Invalid omni account length: expected 32 bytes, got {}",
						address_bytes.len()
					)),
				));
			}

			let omni_account = AccountId::decode(&mut &address_bytes[..]).map_err(|_| {
				error!("Failed to decode AccountId from bytes");
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to decode AccountId from bytes"),
				)
			})?;

			// Query records from storage
			let record_list = ctx.loan_record_storage.query_records(&omni_account, params.nonce);

			// Convert to HashMap keyed by nonce (as string)
			let mut records = HashMap::new();
			for (nonce, record) in record_list {
				records.insert(nonce.to_string(), record);
			}

			Ok(QueryLoanTestResponse { records })
		})
		.expect("Failed to register omni_queryLoanTest method");
}
