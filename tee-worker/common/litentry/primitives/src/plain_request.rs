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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

/// A morphling of itp_types::RsaRequest which stems from teebag::RsaRequest
///
/// Instead of encrypting the TrustedCallSigned with the TEE's shielding key, we encrypt
/// it with a 32-byte ephemeral AES key which is generated from the client's side, and
/// send the encrypted payload together with the AES key encrypted using TEE's shielding key.
///
/// After the enclave gets the request, it will decrypt to get the AES key and use that key
/// to decrypt the payload and decode it to get the real TrustedCall.
///
/// The motivation of having such a struct is:
/// 1. RSA has a limitation of maximum allowed test to be encrypted. In our case, the encoded
///    `TrustedCallSigned` can exceed the limit, AES doesn't have such problem.
///
/// 2. we want to efface the shielding key setup completely to achieve a better UE.
use crate::{Debug, ShardIdentifier, Vec};
use codec::{Decode, Encode};

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct PlainRequest {
	pub shard: ShardIdentifier,
	pub payload: Vec<u8>,
}
