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

use crate::event_handler::EventHandler;
use crate::fetcher::Fetcher;
use crate::sync_checkpoint::SyncCheckpoint;
use executor_core::listener::Listener;
use executor_primitives::{BlockEvent, EventId};
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_signer::key_store::SubstrateKeyStore;
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
	AccountStoreStorage,
	MemberAccountStorage,
> = Listener<
	Fetcher<
		<ChainConfig as Config>::AccountId,
		<ChainConfig as Config>::Header,
		RpcClient,
		RpcClientFactory,
	>,
	SyncCheckpoint,
	CheckpointRepository,
	IntentEventId,
	BlockEvent,
	EventHandler<
		ChainConfig,
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		SubstrateKeyStore,
		RpcClient,
		RpcClientFactory,
		AccountStoreStorage,
		MemberAccountStorage,
	>,
>;
