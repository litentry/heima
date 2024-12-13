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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

extern crate alloc;
extern crate core;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use alloc::vec;
use log::error;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;

#[cfg(feature = "std")]
use std::sync::Mutex;

use itc_direct_rpc_client::{DirectRpcClientFactory, Response, RpcClient, RpcClientFactory};
use itc_direct_rpc_server::{
	response_channel::ResponseChannel, rpc_responder::RpcResponder, RpcConnectionRegistry,
	SendRpcResponse,
};
use itp_rpc::{Id, RpcRequest};
use itp_stf_primitives::types::Hash;
use itp_types::{DirectRequestStatus, TrustedOperationStatus, H256};
use itp_utils::FromHexPrefixed;
use litentry_primitives::BroadcastedRequest;
use log::*;
use std::{
	collections::HashMap,
	string::{String, ToString},
	sync::{
		mpsc::{sync_channel, SyncSender},
		Arc,
	},
	vec::Vec,
};

pub type MaybeRequestIdWithParams = Option<(Hash, Vec<String>)>;

pub trait PeerUpdater {
	fn update(&self, peers: Vec<String>);
}

pub struct DirectRpcBroadcaster<ClientFactory>
where
	ClientFactory: RpcClientFactory,
{
	peers: Mutex<HashMap<String, ClientFactory::Client>>,
	responses_sender: SyncSender<Response>,
	factory: ClientFactory,
}

impl<ClientFactory> DirectRpcBroadcaster<ClientFactory>
where
	ClientFactory: RpcClientFactory,
{
	pub fn new<Registry, ResponseChannelType>(
		peers: &[&str],
		client_factory: ClientFactory,
		rpc_responder: Arc<RpcResponder<Registry, Hash, ResponseChannelType>>,
	) -> Self
	where
		Registry: RpcConnectionRegistry<Hash = Hash> + 'static,
		ResponseChannelType: ResponseChannel<Registry::Connection> + 'static,
	{
		let (responses_sender, responses_receiver) = sync_channel(1000);
		let mut peers_map = HashMap::new();
		for peer in peers {
			match client_factory.create(peer, responses_sender.clone()) {
				Ok(client) => {
					peers_map.insert(peer.to_string(), client);
				},
				Err(e) => log::error!("Could not connect to peer {}, reason: {:?}", peer, e),
			}
		}

		std::thread::spawn(move || {
			while let Ok((id, rpc_return_value)) = responses_receiver.recv() {
				match rpc_return_value.status {
					DirectRequestStatus::TrustedOperationStatus(status, _) => {
						//we need to map Id to hash in order to correlate it with connection
						let hash = match id_to_hash(&id) {
							Some(hash) => hash,
							None => continue,
						};
						info!("Received top broadcast, hash = {:?}", hash);
						match status {
							// this will come from every peer so do not flood the client
							TrustedOperationStatus::Submitted => {
								info!("Status: Submitted");
							},
							// this needs to come before block is imported, otherwise it's going to be ignored because TOP will be removed from the pool after block import
							TrustedOperationStatus::TopExecuted(ref value, force_wait) => {
								info!("Status: TopExecuted");
								match rpc_responder.update_connection_state(
									hash,
									value.clone(),
									force_wait,
								) {
									Ok(_) => {},
									Err(e) => log::error!(
										"Could not set connection {}, reason: {:?}",
										hash,
										e
									),
								};
								if let Err(_e) = rpc_responder.update_status_event(hash, status) {
									error!("Could not update status for {}", &hash)
								};
							},
							_ => {
								info!("Unknown status: {:?}", status);
								//as long as we are waiting let's ignore all status events.
								if !rpc_responder.is_force_wait(hash) {
									if let Err(_e) = rpc_responder.update_status_event(hash, status)
									{
									};
								}
							},
						}
					},

					DirectRequestStatus::Ok
					| DirectRequestStatus::Error
					| DirectRequestStatus::Processing(_) => {
						log::warn!(
							"Got unexpected direct request status: {:?}",
							rpc_return_value.status
						);
					},
				}
			}
		});

		DirectRpcBroadcaster {
			peers: Mutex::new(peers_map),
			responses_sender,
			factory: client_factory,
		}
	}

	fn new_clear_peer_map(&self) -> HashMap<String, ClientFactory::Client> {
		HashMap::new()
	}

	pub fn broadcast(&self, request: BroadcastedRequest) {
		if let Ok(mut peers) = self.peers.lock() {
			let request = RpcRequest {
				jsonrpc: "2.0".to_string(),
				method: request.rpc_method.clone(),
				params: vec![request.payload.clone()],
				id: Id::Text(request.id),
			};
			peers.values_mut().for_each(|peer| {
				if let Err(e) = peer.send(&request) {
					log::warn!("Could not send top to peer reason: {:?}", e);
				}
			});
		}
	}

	fn connect_to(&self, url: &str, peer_list: &mut HashMap<String, ClientFactory::Client>) {
		match self.factory.create(url, self.responses_sender.clone()) {
			Ok(client) => {
				peer_list.insert(url.to_string(), client);
			},
			Err(e) => log::error!("Could not connect to peer {}, reason: {:?}", url, e),
		}
	}
}

pub fn id_to_hash(id: &Id) -> Option<Hash> {
	match id {
		Id::Text(id) => H256::from_hex(id).ok(),
		Id::Number(id) => {
			log::error!("Got response with id {}", id);
			None
		},
	}
}

#[allow(clippy::type_complexity)]
pub fn init<Registry, ResponseChannelType>(
	rpc_responder: Arc<RpcResponder<Registry, Hash, ResponseChannelType>>,
) -> (
	Arc<std::sync::mpsc::SyncSender<BroadcastedRequest>>,
	Arc<DirectRpcBroadcaster<DirectRpcClientFactory>>,
)
where
	Registry: RpcConnectionRegistry<Hash = Hash> + 'static,
	ResponseChannelType: ResponseChannel<Registry::Connection> + 'static,
{
	let (sender, receiver) = std::sync::mpsc::sync_channel::<BroadcastedRequest>(1000);

	let peers = vec![];

	let client_factory = DirectRpcClientFactory {};

	let rpc_broadcaster =
		Arc::new(DirectRpcBroadcaster::new(&peers, client_factory, rpc_responder));
	let return_rpc_broadcaster = rpc_broadcaster.clone();

	std::thread::spawn(move || {
		for received in receiver {
			rpc_broadcaster.broadcast(received);
		}
	});

	(Arc::new(sender), return_rpc_broadcaster)
}

impl<ClientFactory> PeerUpdater for DirectRpcBroadcaster<ClientFactory>
where
	ClientFactory: RpcClientFactory,
{
	// created new map filled with rpc clients connected to peer from the provided list. Reuses existing
	// connections. The list will not containt peers that are unreachable, so following logic will automatically
	// remove all dead connections
	fn update(&self, peers: Vec<String>) {
		log::debug!("Updating peers: {:?}", &peers);
		let mut new_peers_list = self.new_clear_peer_map();
		for peer in peers {
			if let Ok(mut peers) = self.peers.lock() {
				if !peers.contains_key(&peer) {
					log::info!("Adding a peer: {}", peer.clone());
					self.connect_to(&peer, &mut new_peers_list)
				} else {
					log::info!("Reusing existing peer: {}", peer.clone());
					//this is safe as we previously ensured that map contains such key
					let peer_to_move = peers.remove(&peer).unwrap();
					new_peers_list.insert(peer, peer_to_move);
				}
			}
		}
		if let Ok(mut peers) = self.peers.lock() {
			*peers = new_peers_list;
		}
	}
}

#[cfg(test)]
pub mod tests {
	use crate::{DirectRpcBroadcaster, PeerUpdater};
	use alloc::sync::Arc;
	use itc_direct_rpc_client::{Response, RpcClient, RpcClientFactory};
	use itc_direct_rpc_server::{
		mocks::response_channel_mock::ResponseChannelMock,
		rpc_connection_registry::ConnectionRegistry, rpc_responder::RpcResponder,
	};
	use itp_rpc::{Id, RpcRequest, RpcReturnValue};
	use itp_stf_primitives::types::Hash;
	use itp_types::H256;
	use itp_utils::ToHexPrefixed;
	use litentry_primitives::BroadcastedRequest;
	use std::{error::Error, sync::mpsc::SyncSender};

	type TestConnectionToken = u64;
	type TestResponseChannel = ResponseChannelMock<TestConnectionToken>;
	type TestConnectionRegistry = ConnectionRegistry<H256, TestConnectionToken>;

	#[derive(Default)]
	pub struct MockedRpcClient {
		pub sent_requests: u64,
		pub response: Option<(Id, RpcReturnValue)>,
	}

	impl RpcClient for MockedRpcClient {
		fn send(&mut self, _request: &RpcRequest) -> Result<(), Box<dyn Error>> {
			self.sent_requests += 1;
			Ok(())
		}
	}

	impl MockedRpcClient {
		pub fn set_response(&mut self, response: (Id, RpcReturnValue)) {
			self.response = Some(response)
		}
	}

	pub struct MockedRpcClientFactory {}

	impl RpcClientFactory for MockedRpcClientFactory {
		type Client = MockedRpcClient;

		fn create(
			&self,
			_url: &str,
			_response_sink: SyncSender<Response>,
		) -> Result<Self::Client, Box<dyn Error>> {
			Ok(MockedRpcClient::default())
		}
	}

	#[test]
	pub fn creates_initial_peers() {
		//given
		let factory = MockedRpcClientFactory {};
		let connection_registry = Arc::new(TestConnectionRegistry::new());
		let websocket_responder = Arc::new(TestResponseChannel::default());
		let rpc_responder = Arc::new(RpcResponder::new(connection_registry, websocket_responder));

		//when
		let broadcaster: DirectRpcBroadcaster<MockedRpcClientFactory> =
			DirectRpcBroadcaster::new(&["localhost"], factory, rpc_responder);

		//then
		assert_eq!(broadcaster.peers.lock().unwrap().len(), 1);
	}

	#[test]
	pub fn broadcast_sends_to_all_peers() {
		//given
		let factory = MockedRpcClientFactory {};
		let connection_registry = Arc::new(TestConnectionRegistry::new());
		let websocket_responder = Arc::new(TestResponseChannel::default());
		let rpc_responder = Arc::new(RpcResponder::new(connection_registry, websocket_responder));

		let broadcaster: DirectRpcBroadcaster<MockedRpcClientFactory> =
			DirectRpcBroadcaster::new(&["localhost", "localhost2"], factory, rpc_responder);

		//when
		broadcaster.broadcast(BroadcastedRequest {
			id: Hash::random().to_hex(),
			payload: Hash::random().to_hex(),
			rpc_method: "submit_and_broadcast".to_string(),
		});
		broadcaster.broadcast(BroadcastedRequest {
			id: Hash::random().to_hex(),
			payload: Hash::random().to_hex(),
			rpc_method: "submit_and_broadcast".to_string(),
		});

		//then
		let peers = broadcaster.peers.lock().unwrap();
		for peer in peers.iter() {
			assert_eq!(peer.1.sent_requests, 2u64)
		}
	}

	#[test]
	pub fn updates_list_correctly() {
		//given
		let retained_peer = "localhost";
		let added_peer = "localhost3";
		let removed_peer = "localhost2";

		let factory = MockedRpcClientFactory {};
		let connection_registry = Arc::new(TestConnectionRegistry::new());
		let websocket_responder = Arc::new(TestResponseChannel::default());
		let rpc_responder = Arc::new(RpcResponder::new(connection_registry, websocket_responder));

		let broadcaster: DirectRpcBroadcaster<MockedRpcClientFactory> =
			DirectRpcBroadcaster::new(&[retained_peer, removed_peer], factory, rpc_responder);

		//when
		broadcaster.update(vec![retained_peer.to_string(), added_peer.to_string()]);

		//then
		let peers = broadcaster.peers.lock().unwrap();
		assert!(peers.get(retained_peer).is_some());
		assert!(peers.get(added_peer).is_some());
		assert!(peers.get(removed_peer).is_none());
	}
}
