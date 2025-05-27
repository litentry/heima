pub mod solana;

use alloy::{
	primitives::{Address, TxKind, U256},
	rpc::types::{TransactionInput, TransactionRequest},
	sol,
	sol_types::{SolInterface, SolValue},
};
use async_trait::async_trait;
use ethereum_rpc::RpcProvider;
use AccountingContract::AccountingContractCalls;

sol!(
	#[allow(missing_docs)]
	#[sol(rpc)]
	AccountingContract,
	"./abi/AccountingContract.json"
);

#[async_trait]
pub trait AccountingContractApi: Send + Sync {
	async fn execute_pay_out_request(
		&self,
		beneficiary: Address,
		nonce: U256,
		amount: U256,
	) -> Result<(), ()>;

	async fn get_nonce(&self, user: Address) -> Result<U256, ()>;

	async fn get_balance(&self) -> Result<U256, ()>;

	async fn get_address(&self) -> Address;

	async fn get_signer_address(&self) -> Address;
}

pub struct AccountingContractClient<P: RpcProvider<Transaction = TransactionRequest> + Send + Sync>
{
	pub provider: P,
	pub contract_address: Address,
}

impl<P: RpcProvider<Transaction = TransactionRequest> + Send + Sync> AccountingContractClient<P> {
	pub fn new(provider: P, contract_address: Address) -> Self {
		Self { provider, contract_address }
	}
}

#[async_trait]
impl<P: RpcProvider<Transaction = TransactionRequest> + Send + Sync> AccountingContractApi
	for AccountingContractClient<P>
{
	async fn execute_pay_out_request(
		&self,
		beneficiary: Address,
		nonce: U256,
		amount: U256,
	) -> Result<(), ()> {
		let call = AccountingContractCalls::executePayOutRequest(
			AccountingContract::executePayOutRequestCall { beneficiary, nonce, amount },
		);

		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.abi_encode().into()), ..Default::default() },
			..Default::default()
		};

		self.provider.send_transaction(tx).await
	}

	async fn get_nonce(&self, user: Address) -> Result<U256, ()> {
		let call = AccountingContractCalls::getNonce(AccountingContract::getNonceCall { user })
			.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.into()), ..Default::default() },
			..Default::default()
		};
		self.provider.call(tx).await.map_or(Err(()), |nonce| {
			let nonce = U256::abi_decode(&nonce).unwrap();
			Ok(nonce)
		})
	}

	async fn get_balance(&self) -> Result<U256, ()> {
		let call =
			AccountingContractCalls::getBalance(AccountingContract::getBalanceCall {}).abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.into()), ..Default::default() },
			..Default::default()
		};
		self.provider.call(tx).await.map_or(Err(()), |balance| {
			let balance = U256::abi_decode(&balance).unwrap();
			Ok(balance)
		})
	}

	async fn get_address(&self) -> Address {
		self.contract_address
	}

	async fn get_signer_address(&self) -> Address {
		self.provider.get_wallet_address().await.expect("No signer address")
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {
	use crate::AccountingContractApi;
	use crate::Address;
	use crate::U256;
	use async_trait::async_trait;
	use mockall::mock;

	mock! {

		pub AccountingContractClient {}

		#[async_trait]
		impl AccountingContractApi for AccountingContractClient {

			async fn execute_pay_out_request(
				&self,
				beneficiary: Address,
				nonce: U256,
				amount: U256,
			) -> Result<(), ()>;

			async fn get_nonce(&self, user: Address) -> Result<U256, ()>;

			async fn get_balance(&self) -> Result<U256, ()>;

			async fn get_address(&self) -> Address {
				Address::default();
			}

			async fn get_signer_address(&self) -> Address {
				Address::default();
			}
		}
	}
}
