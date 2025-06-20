use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "dstAmount")]
    pub dst_amount: String,
    pub tx: TransactionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas: u64,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    pub chain_id: u64,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub slippage: String,
    pub user_wallet_address: String,
    pub fee_percent: String,
    pub referrer: String,
    pub gas_level: u8,
    pub dex_ids: String
}

impl SwapRequest {

    // TODO, adding protocols is conditional
    pub fn convert_to_query_params(&self) -> Vec<(&'static str, &'_ str)> {
        vec![
            ("amount", &self.amount),
            ("src", &self.from_token_address),
            ("dst", &self.to_token_address),
            ("slippage", &self.slippage),
            ("from", &self.user_wallet_address),
            ("origin", &self.user_wallet_address),
            ("fee", &self.fee_percent),
            ("referrer", &self.referrer),
            ("protocols", &self.dex_ids),
        ]
    }
}