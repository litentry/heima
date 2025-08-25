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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
	#[error("Invalid address format: {address}")]
	InvalidAddress { address: String },

	#[error("Invalid amount format: {amount}")]
	InvalidAmount { amount: String },

	#[error("Invalid chain ID: {chain_id}")]
	InvalidChainId { chain_id: String },

	#[error("Invalid decimal format: {value}")]
	InvalidDecimal { value: String },

	#[error("Invalid hex format: {value}")]
	InvalidHex { value: String },

	#[error("HTTP request failed: {message}")]
	HttpRequest { message: String },

	#[error("JSON serialization/deserialization failed: {message}")]
	JsonSerialization { message: String },

	#[error("Gas calculation failed: {reason}")]
	GasCalculation { reason: String },

	#[error("Transaction data extraction failed: {reason}")]
	TransactionDataExtraction { reason: String },

	#[error("Missing required field: {field}")]
	MissingRequiredField { field: String },

	#[error("Invalid slippage value: {value}")]
	InvalidSlippage { value: String },

	#[error("Invalid gas level: {level}")]
	InvalidGasLevel { level: String },

	#[error("Unsupported trade pool: {pool_name} for chain {chain_id}")]
	UnsupportedTradePool { pool_name: String, chain_id: u64 },

	#[error("Unsupported DEX: {dex_name} for trade pool {pool_name}")]
	UnsupportedDex { dex_name: String, pool_name: String },

	#[error("Amount overflow or underflow")]
	AmountOverflow,

	#[error("System time error: {message}")]
	SystemTime { message: String },

	#[error("Builder validation failed: {message}")]
	BuilderValidation { message: String },

	#[error("Network error on chain {chain_id}: {message}")]
	NetworkWithChain { chain_id: u64, message: String },

	#[error("Network error: {message}")]
	Network { message: String },

	#[error("Internal error: {message}")]
	Internal { message: String },

	#[error("Insufficient balance: required {required}, available {available}")]
	InsufficientBalance { required: String, available: String },

	#[error("Transaction not set: {field}")]
	TransactionFieldNotSet { field: String },

	#[error("Conversion error: {message}")]
	ConversionError { message: String },

	#[error("Unsupported decimals: {decimals}")]
	UnsupportedDecimals { decimals: u8 },

	#[error("Unsupported chain ID: {chain_id}")]
	UnsupportedChainId { chain_id: u64 },
}

impl From<reqwest::Error> for ClientError {
	fn from(err: reqwest::Error) -> Self {
		ClientError::HttpRequest { message: err.to_string() }
	}
}

impl From<serde_json::Error> for ClientError {
	fn from(err: serde_json::Error) -> Self {
		ClientError::JsonSerialization { message: err.to_string() }
	}
}

impl From<hex::FromHexError> for ClientError {
	fn from(err: hex::FromHexError) -> Self {
		ClientError::InvalidHex { value: err.to_string() }
	}
}

impl From<rust_decimal::Error> for ClientError {
	fn from(err: rust_decimal::Error) -> Self {
		ClientError::InvalidDecimal { value: err.to_string() }
	}
}

impl From<std::time::SystemTimeError> for ClientError {
	fn from(err: std::time::SystemTimeError) -> Self {
		ClientError::SystemTime { message: err.to_string() }
	}
}

pub type ClientResult<T> = Result<T, ClientError>;
