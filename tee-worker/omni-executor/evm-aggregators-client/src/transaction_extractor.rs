// Copyright 2020-2025 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::common::GAS_LIMIT;
use crate::errors::{ClientError, ClientResult};
use alloy::primitives::{Address, U256};
use hex::FromHex;
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use std::ops::Mul;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum TransactionGas {
    AsU64(u64),
    AsString(String),
}

pub trait TransactionDataExtractor {
    fn get_data(&self) -> &str;
    fn get_value(&self) -> &str;
    fn get_to_address(&self) -> &str;
    fn get_gas(&self) -> TransactionGas;
}

pub fn extract_transaction_data<T: TransactionDataExtractor>(
    extractor: T,
) -> ClientResult<(Vec<u8>, Address, U256, u64)> {
    let data = hex::decode(extractor.get_data())
        .map_err(|e| ClientError::InvalidHex { value: format!("Transaction data: {}", e) })?;

    let value: U256 = U256::from_str(extractor.get_value())
        .map_err(|_| ClientError::InvalidAmount { amount: extractor.get_value().to_string() })?;

    let to = Address::from_hex(extractor.get_to_address())
        .map_err(|_| ClientError::InvalidAddress { address: extractor.get_to_address().to_string() })?;

    let gas = match extractor.get_gas() {
        TransactionGas::AsU64(gas_val) => Decimal::from_u64(gas_val)
            .ok_or_else(|| ClientError::GasCalculation { reason: "Failed to convert gas to decimal".to_string() })?,
        TransactionGas::AsString(gas_str) => Decimal::from_str(&gas_str)
            .map_err(|_| ClientError::GasCalculation { reason: "Failed to parse gas value".to_string() })?,
    };

    let adjusted_gas = gas.mul(Decimal::new(15, 1)).to_u64()
        .ok_or_else(|| ClientError::GasCalculation { reason: "Failed to convert adjusted gas to u64".to_string() })?;
    
    let final_gas = if adjusted_gas > GAS_LIMIT { GAS_LIMIT } else { adjusted_gas };

    Ok((data, to, value, final_gas))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransactionExtractor {
        data: String,
        value: String,
        to_address: String,
        gas: TransactionGas,
    }

    impl TransactionDataExtractor for MockTransactionExtractor {
        fn get_data(&self) -> &str {
            &self.data
        }

        fn get_value(&self) -> &str {
            &self.value
        }

        fn get_to_address(&self) -> &str {
            &self.to_address
        }

        fn get_gas(&self) -> TransactionGas {
            self.gas.clone()
        }
    }

    #[test]
    fn test_gas_calculation_multiplier() {
        let mock_extractor = MockTransactionExtractor {
            data: "1234".to_string(),
            value: "1000000000000000000".to_string(), // 1 ETH in wei
            to_address: "742d35Cc6641b4Fc7b05cC38f69Cc8D7C2B6B444".to_string(),
            gas: TransactionGas::AsU64(100000), // 100k gas
        };

        let result = extract_transaction_data(mock_extractor).unwrap();
        let calculated_gas = result.3; // gas is the 4th element

        // Original gas: 100,000
        // Expected: 100,000 * 1.5 = 150,000
        let expected_gas = 150000u64;
        
        assert_eq!(calculated_gas, expected_gas, 
            "Gas should be multiplied by 1.5. Expected {}, got {}", 
            expected_gas, calculated_gas);
    }

    #[test]
    fn test_gas_calculation_with_string_input() {
        let mock_extractor = MockTransactionExtractor {
            data: "5678".to_string(),
            value: "2000000000000000000".to_string(), // 2 ETH in wei
            to_address: "742d35Cc6641b4Fc7b05cC38f69Cc8D7C2B6B444".to_string(),
            gas: TransactionGas::AsString("200000".to_string()), // 200k gas as string
        };

        let result = extract_transaction_data(mock_extractor).unwrap();
        let calculated_gas = result.3;

        // Original gas: 200,000
        // Expected: 200,000 * 1.5 = 300,000
        let expected_gas = 300000u64;
        
        assert_eq!(calculated_gas, expected_gas,
            "Gas should be multiplied by 1.5. Expected {}, got {}",
            expected_gas, calculated_gas);
    }

    #[test]
    fn test_gas_calculation_respects_limit() {
        let mock_extractor = MockTransactionExtractor {
            data: "9abc".to_string(),
            value: "3000000000000000000".to_string(), // 3 ETH in wei
            to_address: "742d35Cc6641b4Fc7b05cC38f69Cc8D7C2B6B444".to_string(),
            gas: TransactionGas::AsU64(400000), // 400k gas
        };

        let result = extract_transaction_data(mock_extractor).unwrap();
        let calculated_gas = result.3;

        // Original gas: 400,000
        // Expected after 1.5x: 400,000 * 1.5 = 600,000
        // But GAS_LIMIT is 450,000, so it should be capped
        let expected_gas = GAS_LIMIT; // Should be capped at 450,000
        
        assert_eq!(calculated_gas, expected_gas,
            "Gas should be capped at GAS_LIMIT ({}). Got {}",
            GAS_LIMIT, calculated_gas);
    }
}