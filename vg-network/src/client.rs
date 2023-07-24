use crate::{PeerData, SocketData};

/// I am a client
pub struct ClientData {
    host: PeerData,
}

impl ClientData {
    pub fn new() -> Self {
        Self {
            host: PeerData::new(),
        }
    }

    pub fn poll(&self, socket: &mut SocketData) {}
}
