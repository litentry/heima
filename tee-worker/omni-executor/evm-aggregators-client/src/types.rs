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

use crate::errors::{ClientError, ClientResult};
use alloy::primitives::Address as AlloyAddress;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(AlloyAddress);

impl Address {
    pub fn new(address: AlloyAddress) -> Self {
        Self(address)
    }

    pub fn as_alloy(&self) -> &AlloyAddress {
        &self.0
    }

    pub fn into_alloy(self) -> AlloyAddress {
        self.0
    }

    pub fn to_checksum_string(&self) -> String {
        format!("{:#x}", self.0)
    }
}

impl FromStr for Address {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = AlloyAddress::from_str(s).map_err(|_| ClientError::InvalidAddress {
            address: s.to_string(),
        })?;
        Ok(Self(address))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl From<AlloyAddress> for Address {
    fn from(address: AlloyAddress) -> Self {
        Self(address)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(u64);

impl ChainId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for ChainId {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>().map_err(|_| ClientError::InvalidChainId {
            chain_id: s.to_string(),
        })?;
        Ok(Self(id))
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ChainId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenAmount(Decimal);

impl TokenAmount {
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    pub fn from_wei_string(s: &str) -> ClientResult<Self> {
        let decimal = Decimal::from_str(s).map_err(|_| ClientError::InvalidAmount {
            amount: s.to_string(),
        })?;
        Ok(Self(decimal))
    }

    pub fn value(&self) -> &Decimal {
        &self.0
    }

    pub fn into_decimal(self) -> Decimal {
        self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl FromStr for TokenAmount {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_wei_string(s)
    }
}

impl Display for TokenAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Decimal> for TokenAmount {
    fn from(amount: Decimal) -> Self {
        Self(amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slippage(u32);

impl Slippage {
    pub fn new_bps(bps: u32) -> ClientResult<Self> {
        if bps > 10000 {
            return Err(ClientError::InvalidSlippage {
                value: bps.to_string(),
            });
        }
        Ok(Self(bps))
    }

    pub fn bps(&self) -> u32 {
        self.0
    }

    pub fn to_decimal(&self) -> Decimal {
        Decimal::from(self.0) / Decimal::from(10000u32)
    }

    pub fn to_okx_string(&self) -> String {
        self.to_decimal().to_string()
    }

    pub fn to_inch_string(&self) -> String {
        self.to_decimal().to_string()
    }
}

impl FromStr for Slippage {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bps = s.parse::<u32>().map_err(|_| ClientError::InvalidSlippage {
            value: s.to_string(),
        })?;
        Self::new_bps(bps)
    }
}

impl Display for Slippage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasLevel {
    Slow,
    Average,
    Fast,
}

impl GasLevel {
    pub fn to_okx_string(&self) -> &'static str {
        match self {
            GasLevel::Slow => "slow",
            GasLevel::Average => "average",
            GasLevel::Fast => "fast",
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => GasLevel::Slow,
            2 => GasLevel::Average,
            3 => GasLevel::Fast,
            _ => GasLevel::Average,
        }
    }
}

impl FromStr for GasLevel {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "slow" => Ok(GasLevel::Slow),
            "average" => Ok(GasLevel::Average),
            "fast" => Ok(GasLevel::Fast),
            _ => Err(ClientError::InvalidGasLevel { level: s.to_string() }),
        }
    }
}

impl Display for GasLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_okx_string())
    }
}