use crate::types::{addRootSignerCall, getNonceCall, removeRootSignerCall};
use alloy::primitives::{Address, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::{SolCall, SolValue};
use ethereum_rpc::RpcProvider;
use std::sync::Arc;
use tracing::error;

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
		let result = self
			.rpc_client
			.call(tx)
			.await
			.map_err(|e| error!("Could not get nonce: {:?}", e))?;
		let nonce = U256::abi_decode(&result).map_err(|_| error!("Could not decode nonce"))?;
		Ok(nonce)
	}

	pub async fn add_root_signer(&self, root: Address) -> Result<(), ()> {
		let call_data = addRootSignerCall { root }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}

	pub async fn remove_root_signer(&self, root: Address) -> Result<(), ()> {
		let call_data = removeRootSignerCall { root }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}
}

#[cfg(test)]
pub mod test {
	use crate::SmartWalletClient;
	use alloy::network::EthereumWallet;
	use alloy::primitives::{address, U256};
	use alloy::signers::local::PrivateKeySigner;
	use ethereum_rpc::mocks::MockRpcProvider;
	use ethereum_rpc::AlloyRpcProvider;
	use std::str::FromStr;
	use std::sync::Arc;
	use test_log::test;

	#[test(tokio::test)]
	pub async fn test_get_nonce() {
		let expected_nonce = U256::from(42);
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let mut rpc_client = MockRpcProvider::new();

		// Mock the call response - nonce encoded as U256
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(move |_| Ok(U256::from(42).to_be_bytes_vec()));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let nonce = client.get_nonce().await.unwrap();

		assert_eq!(expected_nonce, nonce);
	}

	#[test(tokio::test)]
	pub async fn test_add_root_signer() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock successful transaction
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok(()));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_ok());
	}

	#[test(tokio::test)]
	pub async fn test_remove_root_signer() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock successful transaction
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok(()));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_ok());
	}

	#[test(tokio::test)]
	pub async fn test_add_root_signer_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock transaction failure
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(()));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_remove_root_signer_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock transaction failure
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(()));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_get_nonce_decode_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let mut rpc_client = MockRpcProvider::new();

		// Mock call with invalid response data
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok(vec![0x12, 0x34])); // Invalid length for U256

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.get_nonce().await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_get_nonce_call_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let mut rpc_client = MockRpcProvider::new();

		// Mock call failure
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(None));

		let client = SmartWalletClient::new(account_address, Arc::new(rpc_client));
		let result = client.get_nonce().await;

		assert!(result.is_err());
	}

	// Integration tests (marked as ignore for manual testing)
	// account_address should point to SmartWallet instance, run `try_full_flow` from entry_point_client.rs first to deploy it and reuse address from logs
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_get_nonce_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let client = SmartWalletClient::new(account_address, rpc_client);

		let nonce = client.get_nonce().await.unwrap();
		println!("Account nonce: {}", nonce);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_add_root_signer_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let root_signer = address!("0x1234567890123456789012345678901234567890");

		// Setup wallet for signing transactions
		let signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));

		let client = SmartWalletClient::new(account_address, rpc_client);
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_ok());
		println!("Successfully added root signer: {}", root_signer);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_remove_root_signer_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let root_signer = address!("0x1234567890123456789012345678901234567890");

		// Setup wallet for signing transactions
		let signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));

		let client = SmartWalletClient::new(account_address, rpc_client);
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_ok());
		println!("Successfully removed root signer: {}", root_signer);
	}
}
