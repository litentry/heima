use itc_peer_top_broadcaster::PeerUpdater;

pub struct PeerUpdaterMock {}

impl PeerUpdater for PeerUpdaterMock {
	fn update(&self, _peers: Vec<String>) {}
}
