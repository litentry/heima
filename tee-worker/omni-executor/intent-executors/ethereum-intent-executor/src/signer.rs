// Copyright 2020-2024 Trust Computing GmbH.
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

use alloy::signers::local::PrivateKeySigner;
use std::str::FromStr;

pub fn get_omni_account_signer() -> PrivateKeySigner {
	PrivateKeySigner::from_str("0x59c6995e998f97a5a0044964f0945389dc9e86dae86c7a8412f4603b6b78690d")
		.unwrap()
}

#[allow(dead_code)]
pub fn get_sponsor_account_signer() -> PrivateKeySigner {
	PrivateKeySigner::from_str("0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a")
		.unwrap()
}
