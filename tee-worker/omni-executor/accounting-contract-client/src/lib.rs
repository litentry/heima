use alloy::{
	primitives::{Address, TxKind, U256},
	rpc::types::{TransactionInput, TransactionRequest},
	sol,
	sol_types::{SolInterface, SolValue},
};
use ethereum_rpc::RpcProvider;
use AccountingContract::AccountingContractCalls;

sol!(
	#[allow(missing_docs)]
	#[sol(rpc)]
	AccountingContract,
	"./abi/AccountingContract.json"
);

pub struct AccountingContractClient<P: RpcProvider<Transaction = TransactionRequest>> {
	pub provider: P,
	pub contract_address: Address,
}

impl<P: RpcProvider<Transaction = TransactionRequest>> AccountingContractClient<P> {
	pub fn new(provider: P, contract_address: Address) -> Self {
		Self { provider, contract_address }
	}

	pub async fn execute_pay_out_request(
		&self,
		beneficiary: Address,
		nonce: U256,
		amount: U256,
	) -> Result<(), ()> {
		let call =
			AccountingContractCalls::executePayOutRequest(AccountingContract::executePayOutRequestCall {
				beneficiary,
				nonce,
				amount,
			});

		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.abi_encode().into()), ..Default::default() },
			..Default::default()
		};

		self.provider.send_transaction(tx).await
	}

	pub async fn get_nonce(&self, user: Address) -> Result<U256, ()> {
		let call = AccountingContractCalls::getNonce(AccountingContract::getNonceCall { user })
			.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.into()), ..Default::default() },
			..Default::default()
		};
		self.provider.call(tx).await.map_or(Err(()), |nonce| {
			let nonce = U256::abi_decode(&nonce, true).unwrap();
			Ok(nonce)
		})
	}
}
