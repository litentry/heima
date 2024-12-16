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

use crate::{AccountId, Hash, Identity, Vec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MemberAccount {
    Public(Identity),
    Private(Vec<u8>, Hash),
}

impl MemberAccount {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public(..))
    }

    pub fn hash(&self) -> Hash {
        match self {
            Self::Public(id) => id.hash(),
            Self::Private(_, h) => *h,
        }
    }
}

impl From<Identity> for MemberAccount {
    fn from(identity: Identity) -> Self {
        Self::Public(identity)
    }
}

pub trait OmniAccountConverter {
    type OmniAccount;
    fn convert(identity: &Identity) -> Self::OmniAccount;
}

pub struct DefaultOmniAccountConverter;

impl OmniAccountConverter for DefaultOmniAccountConverter {
    type OmniAccount = AccountId;
    fn convert(identity: &Identity) -> AccountId {
        identity.to_omni_account()
    }
}

// This type must be kept in sync with the `Permission` type used in the runtime.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OmniAccountPermission {
    All,
    AccountManagement,
    RequestNativeIntent,
    RequestEthereumIntent,
    RequestSolanaIntent,
}
