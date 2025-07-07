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

mod event_handler;
mod fetcher;
mod listener;
mod sync_checkpoint;

use crate::event_handler::EventHandler;
use crate::fetcher::Fetcher;
use crate::listener::ParentchainListener;
use executor_core::listener::Listener;
use executor_core::sync_checkpoint_repository::FileCheckpointRepository;
use executor_storage::StorageDB;
use parentchain_rpc_client::{
	metadata::SubxtMetadataProvider, CustomConfig, SubxtClient, SubxtClientFactory,
};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::oneshot::Receiver;

/// Creates parentchain listener
pub async fn create_listener(
	id: &str,
	handle: Handle,
	ws_rpc_endpoint: &str,
	stop_signal: Receiver<()>,
	_storage_db: Arc<StorageDB>,
	log_path: &str,
) -> Result<
	ParentchainListener<
		SubxtClient<CustomConfig>,
		SubxtClientFactory<CustomConfig>,
		FileCheckpointRepository,
		CustomConfig,
	>,
	(),
> {
	let client_factory: Arc<SubxtClientFactory<CustomConfig>> =
		Arc::new(SubxtClientFactory::new(ws_rpc_endpoint));

	let fetcher = Fetcher::new(client_factory.clone());
	let last_processed_log_repository = FileCheckpointRepository::new(log_path);

	let metadata_provider =
		Arc::new(SubxtMetadataProvider::new(SubxtClientFactory::new(ws_rpc_endpoint)));

	let event_handler = EventHandler::new(metadata_provider);

	Listener::new(id, handle, fetcher, event_handler, stop_signal, last_processed_log_repository)
}
