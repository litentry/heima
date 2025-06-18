use crate::types::getNonceCall;
use alloy::primitives::{Address, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::{SolCall, SolValue};
use ethereum_rpc::RpcProvider;
use std::sync::Arc;

pub struct SmartWalletClient<P: RpcProvider<Transaction = TransactionRequest>> {
	address: Address,
	rpc_client: Arc<P>,
}

impl<P: RpcProvider<Transaction = TransactionRequest>> SmartWalletClient<P> {
	pub fn new(address: Address, rpc_client: Arc<P>) -> Self {
		Self { address, rpc_client }
	}

	pub async fn get_nonce(&self) -> Result<U256, ()> {
		let call_data = getNonceCall {}.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
		let result = self.rpc_client.call(tx).await.map_err(|_| ())?;
		let nonce = U256::abi_decode(&result).map_err(|_| ())?;
		Ok(nonce)
	}
}
