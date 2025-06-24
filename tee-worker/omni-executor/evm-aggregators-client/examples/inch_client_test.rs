use ethers::prelude::{LocalWallet, Signer};
use evm_aggregators_client::inch_client::client::{InchClient, InchSwap};
use evm_aggregators_client::inch_client::types::SwapRequest;
use std::str::FromStr;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let private_key_hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"; // Replace with your actual private key
	let wallet = LocalWallet::from_str(private_key_hex).expect("Invalid private key");
	println!("Wallet address: {:?}", wallet.address());

	let inch_client = InchClient::new("dummy_secret_key", "https://api.1inch.dev");

	let swap_request = SwapRequest {
		chain_id: 56,
		amount: "2000000000000000".to_string(),
		from_token_address: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
		to_token_address: "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".to_string(),
		slippage: "3".to_string(),
		user_wallet_address: wallet.address().to_string(),
		fee_percent: "1".to_string(),
		referrer: "0xA6A027D83e861BAC9C56EcE8a6c094b0939741af".to_string(),
		gas_level: 1,
		dex_ids: "BSC_PANCAKESWAP_V3".to_string(),
	};

	let swap_response = inch_client.swap(56, swap_request).await.unwrap();

	println!("Swap response: {:?}", swap_response.get_transaction_data());
}
