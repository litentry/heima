pub mod solana;

use alloy::{
	primitives::{Address, TxKind, U256},
	rpc::types::{TransactionInput, TransactionRequest},
	sol,
	sol_types::{SolInterface, SolValue},
};
use async_trait::async_trait;
use oe_client_ethereum::RpcProvider;
use AccountingContract::AccountingContractCalls;

sol!(
	#[allow(missing_docs)]
	#[sol(rpc)]
	AccountingContract,
	"./abi/AccountingContract.json"
);

pub trait Plus<RHS = Self> {
	type Output;

	fn plus(self, rhs: RHS) -> Self::Output;
}

impl Plus<u64> for U256 {
	type Output = U256;

	fn plus(self, rhs: u64) -> Self::Output {
		self + U256::from(rhs)
	}
}

impl Plus<u64> for u64 {
	type Output = u64;

	fn plus(self, rhs: u64) -> Self::Output {
		self + rhs
	}
}

#[async_trait]
pub trait AccountingContractApi<A, N>: Send + Sync
where
	A: Send + Sync,
	N: Plus<u64, Output = N> + std::fmt::Debug,
{
	async fn execute_pay_out_request(
		&self,
		beneficiary: A,
		nonce: N,
		amount: U256,
	) -> Result<(), ()>;

	async fn get_nonce(&self, user: A) -> Result<N, ()>;

	async fn get_balance(&self) -> Result<U256, ()>;

	async fn get_address(&self) -> A {
		unimplemented!("Please provide contract address in implementation")
	}

	async fn get_signer_address(&self) -> A {
		unimplemented!("Please provide signer address in implementation")
	}
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
impl<P: RpcProvider<Transaction = TransactionRequest> + Send + Sync>
	AccountingContractApi<Address, U256> for AccountingContractClient<P>
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

		self.provider.send_transaction(tx).await.map_err(|_| ())?;

		Ok(())
	}

	async fn get_nonce(&self, user: Address) -> Result<U256, ()> {
		let call = AccountingContractCalls::getNonce(AccountingContract::getNonceCall { user })
			.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.contract_address)),
			input: TransactionInput { data: Some(call.into()), ..Default::default() },
			..Default::default()
		};
		self.provider.call(tx).await.map_err(|_| ()).and_then(|nonce| {
			let nonce = U256::abi_decode(&nonce).map_err(|_| ())?;
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
		self.provider.call(tx).await.map_err(|_| ()).and_then(|balance| {
			let balance = U256::abi_decode(&balance).map_err(|_| ())?;
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
		impl AccountingContractApi<Address, U256> for AccountingContractClient {

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

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::network::EthereumWallet;
	use alloy::signers::local::PrivateKeySigner;
	use oe_client_ethereum::AlloyRpcProvider;
	use std::str::FromStr;

	#[tokio::test]
	async fn test_mock_server_execute_pay_out_request() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

		let client = create_oe_client_accounting(
			private_key,
			"0x1234567890123456789012345678901234567890",
			Some(mock_url.clone()),
		)
		.await;

		// Test parameters
		let beneficiary: Address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".parse().unwrap();
		let nonce = U256::from(1);
		let amount = U256::from(1000000000000000000u64); // 1 ETH

		let result = client.execute_pay_out_request(beneficiary, nonce, amount).await;
		assert_eq!(result.is_ok(), true);
	}

	#[tokio::test]
	async fn test_mock_server_get_nonce() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

		let client = create_oe_client_accounting(
			private_key,
			"0x1234567890123456789012345678901234567890",
			Some(mock_url.clone()),
		)
		.await;
		let nonce = client
			.get_nonce("0x1234567890123456789012345678901234567890".parse().unwrap())
			.await
			.expect("Fail to get nonce");
		assert_eq!(nonce, U256::from(1)); // nonce = 1

		let client = create_oe_client_accounting(
			private_key,
			"0x1234567890123456789012345678901234567891",
			Some(mock_url),
		)
		.await;
		let nonce = client
			.get_nonce("0x1234567890123456789012345678901234567891".parse().unwrap())
			.await
			.expect("Fail to get nonce");
		assert_eq!(nonce, U256::ZERO); // nonce = 0
	}

	#[tokio::test]
	async fn test_mock_server_get_balance() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

		// has balance
		let client = create_oe_client_accounting(
			private_key,
			"0x1234567890123456789012345678901234567890",
			Some(mock_url.clone()),
		)
		.await;
		let balance = client.get_balance().await.expect("Fail to get balance");
		assert_eq!(balance, U256::from_str("1000000000000000000000").unwrap()); // 1000 ETH

		// has no balance
		let client = create_oe_client_accounting(
			private_key,
			"0x1234567890123456789012345678901234567891",
			Some(mock_url),
		)
		.await;
		let balance = client.get_balance().await.expect("Fail to get balance");
		assert_eq!(balance, U256::ZERO); // 0 ETH
	}

	async fn create_oe_client_accounting(
		private_key: &str,
		contract_address_str: &str,
		mock_url: Option<String>,
	) -> AccountingContractClient<AlloyRpcProvider> {
		let new_mock_url = if let Some(url) = mock_url {
			url
		} else {
			// Start mock server and get dynamic URL
			mock_server::async_run_test_only().await
		};
		let evm_rpc_url = format!("{}/evm/eth", new_mock_url);

		let signer = PrivateKeySigner::from_str(private_key).expect("Invalid private key");
		let wallet = EthereumWallet::from(signer);

		let contract_address: Address = contract_address_str.parse().unwrap();
		let provider = AlloyRpcProvider::new_with_wallet(&evm_rpc_url, wallet);
		AccountingContractClient::new(provider, contract_address)
	}
}
