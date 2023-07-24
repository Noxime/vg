use anyhow::Result;
use matchbox_socket::{
    MessageLoopFuture, MultipleChannels, PeerId, PeerState, WebRtcChannel, WebRtcSocket,
    WebRtcSocketBuilder,
};
use tracing::{debug, trace};

use crate::{
    message::{Delivery, Message},
    Flags,
};

pub struct Socket {
    socket: WebRtcSocket<MultipleChannels>,
    reliable: WebRtcChannel,
    ordered: WebRtcChannel,
    unreliable: WebRtcChannel,
    /// Does this socket appear to have Host role assigned to it
    is_host: bool,
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
                reliable: socket.take_channel(0).unwrap(),
                ordered: socket.take_channel(1).unwrap(),
                unreliable: socket.take_channel(2).unwrap(),
                socket,
                is_host: false,
            },
            driver,
        )
    }

    pub fn send(&mut self, msg: &impl Message) -> Result<()> {
        let delivery = Message::delivery(msg);
        let bytes = Message::serialize(msg)?;
        let packet = bytes.into_boxed_slice();

        trace!(?delivery, len = packet.len(), "Send");

        let channel = match delivery {
            Delivery::Reliable => &mut self.reliable,
            Delivery::Ordered => &mut self.ordered,
            Delivery::Unreliable => &mut self.unreliable,
        };

        // Send to all connected peers
        for peer in self.socket.connected_peers() {
            channel.send(packet.clone(), peer);
        }

        Ok(())
    }

    pub fn poll(&mut self) {
        for (peer, state) in self.socket.update_peers() {
            debug!(?peer, ?state, "Peer state change");
            self.update_flags(peer, state);
        }

        for (peer, packet) in self.reliable.receive() {
            debug!(?peer, ?packet, "Reliable");
        }
        for (peer, packet) in self.unreliable.receive() {
            debug!(?peer, ?packet, "Unreliable");
        }
    }

    /// Looks for magic flag events
    pub fn update_flags(&mut self, peer: PeerId, state: PeerState) {
        if state != PeerState::Disconnected {
            return;
        }

        let Ok(flags) = Flags::decode(peer) else { return };

        // Unpack bits
        debug!(?flags, "Received flags");
        self.is_host = flags.is_host;
    }
}
