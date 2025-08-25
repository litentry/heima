use ethers::prelude::{LocalWallet, Signer};
use evm_aggregators_client::okx_client::client::{OkxClient, OkxSwap};
use evm_aggregators_client::okx_client::types::SwapRequest;
use std::str::FromStr;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let private_key_hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"; // Replace with your actual private key
	let wallet = LocalWallet::from_str(private_key_hex).expect("Invalid private key");
	println!("Wallet address: {:?}", wallet.address());

	let okx_client = OkxClient::new("dummy_secret_key", "https://www.okx.com");

	let swap_request = SwapRequest {
		chain_id: "56".to_string(),
		amount: "2000000000000000".to_string(),
		from_token_address: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
		to_token_address: "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".to_string(),
		slippage: "0.03".to_string(),
		user_wallet_address: wallet.address().to_string(),
		fee_percent: "1".to_string(),
		from_token_referrer_wallet_address: "0xA6A027D83e861BAC9C56EcE8a6c094b0939741af"
			.to_string(),
		to_token_referrer_wallet_address: "".to_string(),
		gas_level: "slow".to_string(),
		dex_ids: "200".to_string(),
		auto_slippage: false,
	};

	let swap_response = okx_client.swap(swap_request).await.unwrap();

	println!("Swap response: {:?}", swap_response.get_transaction_data());
}
