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

use crate::event_handler::IntentEventHandler;
use crate::fetcher::Fetcher;
use crate::key_store::SubstrateKeyStore;
use crate::metadata::SubxtMetadataProvider;
use crate::primitives::SyncCheckpoint;
use crate::primitives::{BlockEvent, EventId};
use executor_core::listener::Listener;
use subxt::Metadata;
use subxt_core::Config;

pub type IntentEventId = EventId;

pub type ParentchainListener<
	RpcClient,
	RpcClientFactory,
	CheckpointRepository,
	ChainConfig,
	EthereumIntentExecutor,
	SolanaIntentExecutor,
> = Listener<
	Fetcher<<ChainConfig as Config>::AccountId, RpcClient, RpcClientFactory>,
	SyncCheckpoint,
	CheckpointRepository,
	IntentEventId,
	BlockEvent,
	IntentEventHandler<
		ChainConfig,
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		SubstrateKeyStore,
		RpcClient,
		RpcClientFactory,
	>,
>;
