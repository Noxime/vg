mod client;
mod flags;
mod message;
mod server;
mod socket;

use client::ClientData;
pub use flags::Flags;
use instant::{Duration, Instant};
use matchbox_socket::{MessageLoopFuture, PeerId, PeerState, WebRtcSocketBuilder};
use message::Delivery;
use server::HostData;
pub use socket::SocketData;
use tracing::debug;

pub struct Socket {
    socket: SocketData,
    role: RoleData,
}

pub enum RoleData {
    Host(HostData),
    Client(ClientData),
}

impl Socket {
    pub fn new(url: &str) -> (Socket, MessageLoopFuture) {
        let (mut socket, driver) = WebRtcSocketBuilder::new(url)
            .reconnect_attempts(None)
            .add_channel(Delivery::Reliable.as_config())
            .add_channel(Delivery::Ordered.as_config())
            .add_channel(Delivery::Unreliable.as_config())
            .build();

        (
            Socket {
                socket: SocketData {
                    reliable: socket.take_channel(0).unwrap(),
                    ordered: socket.take_channel(1).unwrap(),
                    unreliable: socket.take_channel(2).unwrap(),
                    socket,
                },
                role: RoleData::Host(HostData::new()),
            },
            driver,
        )
    }

    /// Process network messages and update socket state
    pub fn poll(&mut self) {
        for (peer, state) in self.socket.socket.update_peers() {
            debug!(?peer, ?state, "Peer state change");
            self.update_flags(peer, state);
        }

        match &mut self.role {
            RoleData::Host(data) => {
                data.poll(&mut self.socket);
            }
            RoleData::Client(data) => {
                data.poll(&mut self.socket);
            }
        }
    }

    /// Looks for magic flag events
    fn update_flags(&mut self, peer: PeerId, state: PeerState) {
        if state != PeerState::Disconnected {
            return;
        }

        // Short out if not actual peer data (magic mismatch etc)
        let Ok(flags) = Flags::decode(peer) else { return };

        debug!(?flags, "Received flags");

        // We are not the room host
        if !flags.is_host {
            debug!("Assigned client role");
            self.role = RoleData::Client(ClientData::new());
        } else {
            debug!("Assigned host role");
        }
    }
}

const PING_TIMEOUT: Duration = Duration::from_secs(5);

pub struct PeerData {
    /// Timestamp of last sent ping
    ping_send: Instant,
}

impl PeerData {
    pub fn new() -> Self {
        Self {
            ping_send: Instant::now() - PING_TIMEOUT,
        }
    }
}
