use std::str::FromStr;
use alloy::primitives::{Address, U256};
use hex::FromHex;
use log::error;
use rust_decimal::prelude::{Decimal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "dstAmount")]
    pub dst_amount: String,
    pub tx: TransactionData,
}

impl SwapResponse {
    pub fn get_transaction_data(self) -> Result<(Vec<u8>, Address, U256), ()> {
        let data = hex::decode(&self.tx.data).map_err(|e| {
           error!("Failed to decode transaction data: {}", e);
        })?;
        let value: U256 = U256::from_str(&self.tx.value).map_err(|e| {
            error!("Failed to deserialize to u256: {}", e);
        })?;
        let to = Address::from_hex(&self.tx.to).map_err(|e| {
            error!("Failed to decode hex to address: {}", e)
        })?;

        Ok((data, to, value))
    }
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

#[derive(Debug, Serialize, Deserialize, Default)]
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

pub fn convert_slippage_to_inch(slippage: u32) -> String {
    // inch max 50% slippage
    if slippage > 5000 {
        return "50".to_string();
    }

    Decimal::from(slippage)
        .checked_div(Decimal::from(100))
        .map(|d| d.normalize().to_string())
        .unwrap_or_else(|| "0".to_string())
}

