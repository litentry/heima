use std::default::Default;
use evm_aggregators_client::common::KYBER_SWAP_DEX_ID_MAP;
use evm_aggregators_client::kyber_client::client::{KyberClient, KyberSwap};
use evm_aggregators_client::kyber_client::types::{GetSwapRouteRequest, SwapRequest};
use ethers::signers::LocalWallet;
use std::str::FromStr;
use ethers::prelude::Signer;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
   let private_key_hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"; // Replace with your actual private key
   let wallet = LocalWallet::from_str(private_key_hex).expect("Invalid private key");
   println!("Wallet address: {:?}", wallet.address());

   let kyber_client = KyberClient::new("dummy_secret_key");
   let swap_route_request = GetSwapRouteRequest {
      chain_id: 56_u64,
      amount:  "1000000000000000".to_string(),
      from_token_address: "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
      to_token_address: "0x50c5725949a6f0c72e6c4a641f24049a917db0cb".to_string(),
      fee_bps: "100".to_string(),
      referrer: "0xA6A027D83e861BAC9C56EcE8a6c094b0939741af".to_string(),
      dex_ids: "uniswapv3".to_string(),
      is_from_token_referrer: true,
      ..Default::default()
   };

   let swap_route_response = kyber_client.get_swap_route(
      swap_route_request
   ).await.unwrap();

   let swap_request = SwapRequest {
      route_summary: swap_route_response.route_summary,
      sender: Some(wallet.address().to_string()),
      recipient: Some(wallet.address().to_string()),
      deadline: Some(0),
      slippage_bps: Some(100),
      enable_gas_estimation: Some(true),
      ignore_capped_slippage: Some(true),
          ..Default::default()
   };

   let swap_response = kyber_client
       .swap(swap_request).await.unwrap();
   
   println!("Swap response: {:?}", swap_response.get_transaction_data());
}