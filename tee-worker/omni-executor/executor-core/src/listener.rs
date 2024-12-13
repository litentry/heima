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

use log::error;
use std::fmt::Debug;
use std::{marker::PhantomData, thread::sleep, time::Duration};
use tokio::{runtime::Handle, sync::oneshot::Receiver};

use crate::event_handler::{Error, EventHandler};
use crate::fetcher::{EventsFetcher, LastFinalizedBlockNumFetcher};
use crate::primitives::GetEventId;
use crate::sync_checkpoint_repository::{Checkpoint, CheckpointRepository};

/// Component, used to listen to chain and execute requested intents
/// Requires specific implementations of:
/// `Fetcher` - used to fetch data from chain
/// `IntentExecutor` - used to execute intents on target chain
/// `CheckpointRepository` - used to store listener's progress
/// `EventId` - represents chain event id
/// `BlockEvent` - represents chain event
pub struct Listener<
	Fetcher,
	Checkpoint,
	CheckpointRepository,
	BlockEventId,
	BlockEvent,
	IntentEventHandler,
> {
	id: String,
	handle: Handle,
	fetcher: Fetcher,
	intent_event_handler: IntentEventHandler,
	stop_signal: Receiver<()>,
	checkpoint_repository: CheckpointRepository,
	_phantom: PhantomData<(Checkpoint, BlockEventId, BlockEvent)>,
}

impl<
		EventId: Into<CheckpointT> + Clone + Debug,
		BlockEventT: GetEventId<EventId>,
		Fetcher: LastFinalizedBlockNumFetcher + EventsFetcher<EventId, BlockEventT>,
		CheckpointT: PartialOrd + Checkpoint + From<u64>,
		CheckpointRepositoryT: CheckpointRepository<CheckpointT>,
		IntentEventHandler: EventHandler<BlockEventT>,
	> Listener<Fetcher, CheckpointT, CheckpointRepositoryT, EventId, BlockEventT, IntentEventHandler>
{
	pub fn new(
		id: &str,
		handle: Handle,
		fetcher: Fetcher,
		intent_event_handler: IntentEventHandler,
		stop_signal: Receiver<()>,
		last_processed_log_repository: CheckpointRepositoryT,
	) -> Result<Self, ()> {
		Ok(Self {
			id: id.to_string(),
			handle,
			fetcher,
			intent_event_handler,
			stop_signal,
			checkpoint_repository: last_processed_log_repository,
			_phantom: PhantomData,
		})
	}

	/// Start syncing. It's a long-running blocking operation - should be started in dedicated thread.
	pub fn sync(&mut self, start_block: u64) {
		log::info!("Starting {} network sync, start block: {}", self.id, start_block);
		let mut block_number_to_sync = if let Some(ref checkpoint) =
			self.checkpoint_repository.get().expect("Could not read checkpoint")
		{
			if checkpoint.just_block_num() {
				// let's start syncing from next block as we processed previous fully
				checkpoint.get_block_num() + 1
			} else {
				// block processing was interrupted, so we have to process last block again
				// but currently processed logs will be skipped
				checkpoint.get_block_num()
			}
		} else {
			start_block
		};
		log::debug!("Starting sync from {:?}", block_number_to_sync);

		'main: loop {
			if self.stop_signal.try_recv().is_ok() {
				break;
			}

			let maybe_last_finalized_block =
				match self.handle.block_on(self.fetcher.get_last_finalized_block_num()) {
					Ok(maybe_block) => maybe_block,
					Err(_) => {
						log::info!("Could not get last finalized block number");
						sleep(Duration::from_secs(1));
						continue;
					},
				};

			let last_finalized_block = match maybe_last_finalized_block {
				Some(v) => v,
				None => {
					log::info!(
						"Waiting for finalized block, block to sync {}",
						block_number_to_sync
					);
					sleep(Duration::from_secs(1));
					continue;
				},
			};

			log::trace!(
				"Last finalized block: {}, block to sync {}",
				last_finalized_block,
				block_number_to_sync
			);

			//we know there are more block waiting for sync so let's skip sleep
			let fast = match last_finalized_block.checked_sub(block_number_to_sync) {
				Some(v) => v > 1,
				None => false,
			};

			let mut sync_error = false;

			if last_finalized_block >= block_number_to_sync {
				log::info!("Syncing block: {}", block_number_to_sync);
				match self.handle.block_on(self.fetcher.get_block_events(block_number_to_sync)) {
					Ok(events) => {
						for event in events {
							let event_id = event.get_event_id().clone();
							if let Some(ref checkpoint) =
								self.checkpoint_repository.get().expect("Could not read checkpoint")
							{
								if checkpoint.lt(&event.get_event_id().clone().into()) {
									log::info!("Handling event: {:?}", event_id);
									if let Err(e) = self
										.handle
										.block_on(self.intent_event_handler.handle(event))
									{
										log::error!("Could not handle event: {:?}", e);
										match e {
											Error::NonRecoverableError => {
												error!("Non-recoverable intent handling error, event: {:?}", event_id);
												break 'main;
											},
											Error::RecoverableError => {
												error!(
												"Recoverable intent handling error, event: {:?}",
												event_id
											);
												continue 'main;
											},
										}
									}
								} else {
									log::debug!("Skipping event");
								}
							} else {
								log::info!("Handling event: {:?}", event_id);
								if let Err(e) =
									self.handle.block_on(self.intent_event_handler.handle(event))
								{
									log::error!("Could not handle event: {:?}", e);
									match e {
										Error::NonRecoverableError => {
											error!(
											"Non-recoverable intent handling error, event: {:?}",
											event_id
										);
											break 'main;
										},
										Error::RecoverableError => {
											error!(
												"Recoverable intent handling error, event: {:?}",
												event_id
											);
											continue 'main;
										},
									}
								}
							}
							self.checkpoint_repository
								.save(event_id.into())
								.expect("Could not save checkpoint");
						}
						// we processed block completely so store new checkpoint
						self.checkpoint_repository
							.save(CheckpointT::from(block_number_to_sync))
							.expect("Could not save checkpoint");
						log::info!("Finished syncing block: {}", block_number_to_sync);
						block_number_to_sync += 1;
					},
					Err(e) => {
						log::error!("Could not get block {} events: {:?}", block_number_to_sync, e);
						sync_error = true;
					},
				}
			} else {
				log::trace!("Block: {} not yet finalized", block_number_to_sync);
			}

			if !fast || sync_error {
				sleep(Duration::from_secs(1))
			} else {
				log::trace!("Fast sync skipping 1s wait");
			}
		}
	}
}
