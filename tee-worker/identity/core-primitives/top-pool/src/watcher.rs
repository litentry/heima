// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Extrinsics status updates.

extern crate alloc;
use crate::primitives::TxHash;
use alloc::{string::String, sync::Arc, vec::Vec};

use itc_direct_rpc_server::{DirectRpcError, SendRpcResponse};
use itp_types::{BlockHash as SidechainBlockHash, TrustedOperationStatus};
use log::*;

/// Extrinsic watcher.
///
/// Represents a stream of status updates for particular extrinsic.
#[derive(Debug)]
pub struct Watcher<S> {
	//receiver: TracingUnboundedReceiver<TrustedOperationStatus<H, BH>>,
	hash: TxHash,
	is_in_block: bool,
	rpc_response_sender: Arc<S>,
}

impl<S> Watcher<S>
where
	S: SendRpcResponse<Hash = TxHash>,
{
	/// Returns the operation hash.
	pub fn hash(&self) -> &TxHash {
		&self.hash
	}

	pub fn new_watcher(hash: TxHash, rpc_response_sender: Arc<S>) -> Self {
		Watcher { hash, is_in_block: false, rpc_response_sender }
	}

	/// TrustedOperation became ready.
	pub fn ready(&mut self) {
		self.send(TrustedOperationStatus::Ready)
	}

	/// TrustedOperation was moved to future.
	pub fn future(&mut self) {
		self.send(TrustedOperationStatus::Future)
	}

	/// Some state change (perhaps another extrinsic was included) rendered this extrinsic invalid.
	pub fn usurped(&mut self) {
		//self.send(TrustedOperationStatus::Usurped(hash));
		self.send(TrustedOperationStatus::Usurped);
		self.is_in_block = true;
	}

	/// Extrinsic has been included in block with given hash.
	pub fn in_block(&mut self, block_hash: SidechainBlockHash) {
		self.send(TrustedOperationStatus::InSidechainBlock(block_hash));
		self.is_in_block = true;
	}

	/// Extrinsic has been finalized by a finality gadget.
	pub fn finalized(&mut self) {
		//self.send(TrustedOperationStatus::Finalized(hash));
		self.send(TrustedOperationStatus::Finalized);
		self.is_in_block = true;
	}

	/// The block this extrinsic was included in has been retracted
	pub fn finality_timeout(&mut self) {
		//self.send(TrustedOperationStatus::FinalityTimeout(hash));
		self.send(TrustedOperationStatus::FinalityTimeout);
		self.is_in_block = true;
	}

	/// The block this extrinsic was included in has been retracted
	pub fn retracted(&mut self) {
		//self.send(TrustedOperationStatus::Retracted(hash));
		self.send(TrustedOperationStatus::Retracted);
	}

	/// Extrinsic has been marked as invalid by the block builder.
	pub fn invalid(&mut self) {
		self.send(TrustedOperationStatus::Invalid);
		// we mark as finalized as there are no more notifications
		self.is_in_block = true;
	}

	/// TrustedOperation has been dropped from the pool because of the limit.
	pub fn dropped(&mut self) {
		self.send(TrustedOperationStatus::Dropped);
		self.is_in_block = true;
	}

	/// The extrinsic has been broadcast to the given peers.
	pub fn broadcast(&mut self, _peers: Vec<String>) {
		//self.send(TrustedOperationStatus::Broadcast(peers))
		self.send(TrustedOperationStatus::Broadcast)
	}

	/// The extrinsic has been executed.
	pub fn top_executed(&mut self, response: &[u8], force_wait: bool) {
		self.send(TrustedOperationStatus::TopExecuted(response.to_vec(), force_wait))
	}

	/// Returns true if the are no more listeners for this extrinsic or it was finalized.
	pub fn is_done(&self) -> bool {
		self.is_in_block // || self.receivers.is_empty()
	}

	fn send(&mut self, status: TrustedOperationStatus) {
		if let Err(e) = self.rpc_response_sender.update_status_event(*self.hash(), status) {
			match e {
				DirectRpcError::InvalidConnectionHash => {
					warn!("Client connection interrupted while sending status update");
				},
				_ => error!("Failed to send status update to RPC client: {:?}", e),
			}
		}
	}

	// Litentry: set the new rpc response value and force_wait flag
	pub fn update_connection_state(&mut self, encoded_value: Vec<u8>, force_wait: bool) {
		if let Err(e) = self.rpc_response_sender.update_connection_state(
			*self.hash(),
			encoded_value,
			force_wait,
		) {
			warn!("failed to update connection state: {:?}", e);
		}
	}

	// Litentry: swap the old hash with the new one in rpc connection registry
	pub fn swap_rpc_connection_hash(&self, new_hash: TxHash) {
		if let Err(e) = self.rpc_response_sender.swap_hash(*self.hash(), new_hash) {
			warn!("failed to swap rpc connection hash: {:?}", e);
		}
	}
}

/*  /// Sender part of the watcher. Exposed only for testing purposes.
#[derive(Debug)]
pub struct Sender<H, BH> {
	//receivers: Vec<TracingUnboundedSender<TrustedOperationStatus<H, BH>>>,
	//receivers: Vec<H>,
	is_in_block: bool,
}
 */
/* impl<H> Default for Watcher<H> {
	fn default() -> Self {
		Watcher {
			//receivers: Default::default(),
			hash: ,
			is_in_block: false,
		}
	}
}  */

/* impl<H: Clone, BH: Clone> Sender<H, BH> {
	/// Add a new watcher to this sender object.

} */
