use std::collections::HashMap;

use matchbox_socket::PeerId;

use crate::{PeerData, SocketData};

/// I am a server
pub struct HostData {
    clients: HashMap<PeerId, PeerData>,
}

impl HostData {
    pub fn new() -> HostData {
        HostData {
            clients: Default::default(),
        }
    }

    pub fn poll(&self, socket: &mut SocketData) {}
}
