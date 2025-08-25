use alloy::network::{EthereumWallet, TxSigner};
use alloy::primitives::{Address, Bytes, Signature, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use async_trait::async_trait;
use std::str::FromStr;
use tracing::{debug, error};

use crate::{AlloyRpcProvider, RpcProvider};

#[async_trait]
pub trait EthereumClient: Send + Sync {
	async fn transfer(
		&self,
		to: &str,
		value: U256,
		signer: Box<dyn TxSigner<Signature> + Send + Sync>,
	) -> Result<String, ()>;

	async fn transfer_erc20(
		&self,
		to: &str,
		value: U256,
		contract_address: &str,
		signer: Box<dyn TxSigner<Signature> + Send + Sync>,
	) -> Result<String, ()>;

	async fn construct_approve_erc20_tx(
		&self,
		spender: Address,
		value: U256,
		contract_address: Address,
		nonce: u64,
	) -> Result<TransactionRequest, ()>;
}

pub struct EthereumRpcClient {
	rpc_provider: AlloyRpcProvider,
}

impl EthereumRpcClient {
	pub fn new(rpc_url: &str) -> Self {
		let rpc_provider = AlloyRpcProvider::new(rpc_url);
		Self { rpc_provider }
	}
}

#[async_trait]
impl EthereumClient for EthereumRpcClient {
	async fn transfer(
		&self,
		to: &str,
		value: U256,
		signer: Box<dyn TxSigner<Signature> + Send + Sync>,
	) -> Result<String, ()> {
		let to_address = Address::from_str(to).map_err(|err| {
			error!("Could not parse address: {:?}", err);
		})?;

		let tx = TransactionRequest {
			to: Some(TxKind::from(to_address)),
			value: Some(value),
			..Default::default()
		};

		let wallet = EthereumWallet::new(signer);

		let tx_signature = self.rpc_provider.send_transaction_with_wallet(&wallet, tx).await?;

		debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			wallet.default_signer().address(),
			to_address
		);
		debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature)
	}

	async fn transfer_erc20(
		&self,
		to: &str,
		value: U256,
		contract_address: &str,
		signer: Box<dyn TxSigner<Signature> + Send + Sync>,
	) -> Result<String, ()> {
		let to_address = Address::from_str(to).map_err(|err| {
			error!("Could not parse address: {:?}", err);
		})?;

		// ERC20 transfer function signature: transfer(address,uint256)
		// The first 4 bytes of the keccak256 hash of "transfer(address,uint256)"
		let transfer_function_signature = &hex_literal::hex!("a9059cbb");

		// Encode the function call with the recipient address and amount
		let mut data = Vec::with_capacity(4 + 32 + 32);
		data.extend_from_slice(transfer_function_signature);
		// Pad the address to 32 bytes (required by EVM ABI encoding)
		let mut address_bytes = [0u8; 32];
		// Note: address only takes up the last 20 bytes
		address_bytes[12..32].copy_from_slice(to_address.as_slice());
		data.extend_from_slice(&address_bytes);
		data.extend_from_slice(&value.to_be_bytes::<32>());

		let contract_address = Address::from_str(contract_address).map_err(|err| {
			error!("Could not parse address: {:?}", err);
		})?;

		let tx = TransactionRequest {
			to: Some(TxKind::from(contract_address)),
			input: TransactionInput::from(Bytes::from(data)), // Set the encoded function call as input data
			..Default::default()
		};

		let wallet = EthereumWallet::new(signer);

		let tx_signature = self.rpc_provider.send_transaction_with_wallet(&wallet, tx).await?;

		debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			wallet.default_signer().address(),
			contract_address
		);
		debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature)
	}

	async fn construct_approve_erc20_tx(
		&self,
		to: Address,
		value: U256,
		contract_address: Address,
		nonce: u64,
	) -> Result<TransactionRequest, ()> {
		// ERC20 approve function signature: approve(address,uint256)
		let approve_function_signature = &hex_literal::hex!("095ea7b3");

		// Encode the function call with the spender address and amount
		let mut data = Vec::with_capacity(4 + 32 + 32);
		data.extend_from_slice(approve_function_signature);
		let mut address_bytes = [0u8; 32];
		address_bytes[12..32].copy_from_slice(to.as_slice());
		data.extend_from_slice(&address_bytes);
		data.extend_from_slice(&value.to_be_bytes::<32>());

		let tx = TransactionRequest {
			to: Some(TxKind::from(contract_address)),
			input: TransactionInput::from(Bytes::from(data)),
			nonce: Some(nonce),
			..Default::default()
		};

		Ok(tx)
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {

	use crate::client::EthereumClient;
	use alloy::network::TxSigner;
	use alloy::primitives::{Address, Signature, U256};
	use alloy::rpc::types::TransactionRequest;
	use async_trait::async_trait;
	use mockall::mock;

	mock! {
		pub EthereumRpcClient {}

		#[async_trait]
		impl EthereumClient for EthereumRpcClient {

			#[mockall::concretize]
			async fn transfer(
				&self,
				to: &str,
				value: U256,
				signer: Box<dyn TxSigner<Signature> + Send + Sync>,
			) -> Result<String, ()>;

			#[mockall::concretize]
			async fn transfer_erc20(
				&self,
				to: &str,
				value: U256,
				contract_address: &str,
				signer: Box<dyn TxSigner<Signature> + Send + Sync>,
			) -> Result<String, ()>;

			#[mockall::concretize]
			async fn construct_approve_erc20_tx(
				&self,
				spender: Address,
				value: U256,
				contract_address: Address,
				nonce: u64,
			) -> Result<TransactionRequest, ()>;
		}

	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy::network::TxSigner;
	use alloy::signers::local::PrivateKeySigner;
	use std::str::FromStr;

	#[tokio::test]
	async fn test_mock_server_transfer() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let evm_rpc_url = format!("{}/evm/eth", mock_url);

		// Create a test wallet and signer
		let private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
		let signer = PrivateKeySigner::from_str(private_key).expect("Invalid private key");
		let signer_box: Box<dyn TxSigner<Signature> + Send + Sync> = Box::new(signer);

		// Create Ethereum client
		let client = EthereumRpcClient::new(&evm_rpc_url);

		// Test parameters
		let to_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
		let amount = U256::from(1000000000000000000u64); // 1 ETH

		// Execute transfer
		let result = client.transfer(to_address, amount, signer_box).await;
		assert_eq!(result.is_ok(), true);
	}

	#[tokio::test]
	async fn test_mock_server_transfer_erc20() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let evm_rpc_url = format!("{}/evm/bsc", mock_url);

		// Create a test wallet and signer
		let private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
		let signer = PrivateKeySigner::from_str(private_key).expect("Invalid private key");
		let signer_box: Box<dyn TxSigner<Signature> + Send + Sync> = Box::new(signer);

		// Create Ethereum client
		let client = EthereumRpcClient::new(&evm_rpc_url);

		// Test parameters
		let to_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
		let token_address = "0xA0b86a33E6441e6e80D0c4C6C7527d72e1d0c4e6";
		let amount = U256::from(1000000000000000000u64); // 1 token (assuming 18 decimals)

		// Execute ERC20 transfer
		let result = client.transfer_erc20(to_address, amount, token_address, signer_box).await;
		assert_eq!(result.is_ok(), true);
	}
}
