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

// By the time of this migration on Litentry 9220 (polkadot stable2407)
// The current        storage version:      pallet version:
// scheduler:                      -                     0
// multisig:                       -                     0
// preimage:                       -                     0
// balances:                       -                     0
// democracy:                      -                     0
// bounties:                       -                     0
// xcmpQueue:                      -                     3
// polkadotXcm:                    -                     0
// developerCommittee              -                     0
// developerCommitteeMembership    -                     0
// transactionPayment:             V1Ancient             0
// vesting:                        V1                    0

// Our target         storage version:      pallet version: (stable2407)
// scheduler:                      -                     0 => 4
// multisig:                       -                     0 => 1
// preimage:                       -                     0 => 1
// balances:                       -                     0 => 1
// democracy:                      -                     0 => 1
// bounties:                       -                     0 => 4
// xcmpQueue:                      -                     3 => 5
// polkadotXcm:                    -                     0 => 1
// developerCommittee              -                     0 => 4
// developerCommitteeMembership    -                     0 => 4
// transactionPayment:             V1Ancient => V2       0   (it is built by genesis, so maybe no fix is fine)
// vesting:                        V1                    0

// In try-runtime, current implementation, the storage version is not checked,
// Pallet version is used instead.

use frame_support::traits::OnRuntimeUpgrade;
use sp_std::marker::PhantomData;
