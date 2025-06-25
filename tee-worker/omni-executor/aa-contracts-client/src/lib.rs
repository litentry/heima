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

mod entry_point_client;
mod smart_wallet_client;
mod types;
mod utils;

pub use entry_point_client::{prepare_factory_init_code, EntryPointClient};
pub use smart_wallet_client::SmartWalletClient;
pub use types::PackedUserOperation;
pub use utils::calculate_smart_account_address;
