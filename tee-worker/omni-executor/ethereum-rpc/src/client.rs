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
}

#[cfg(feature = "mocks")]
pub mod mocks {

	use crate::client::EthereumClient;
	use alloy::network::TxSigner;
	use alloy::primitives::{Signature, U256};
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
		}

	}
}
