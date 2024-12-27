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

// By the time of this migration on Paseo 9221 (polkadot stable2407)
// The current storage version:      pallet version:
// xcmpQueue:                -                     3
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// Our target  storage version:      pallet version: (stable2407)
// xcmpQueue:                -                     3 => 5
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// In try-runtime, current implementation, the storage version is not checked,
// Pallet version is used instead.

pub type Migrations<Runtime> = (
	// V3 to V4
	// XCMP QueueConfig has different default value
	// Migration targeting at changing storage value to new default value if old value matched
	// Our current Paseo setting has already hard-coded
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4<Runtime>,
	// V4 to V5
	// Did nothing to storage
	// Just checking MaxActiveOutboundChannels is not exceeded
	// Our current Paseo Storage is 0
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
);