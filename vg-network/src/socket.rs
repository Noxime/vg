use std::collections::HashMap;

use anyhow::{anyhow, Result};
use instant::{Duration, Instant};
use matchbox_socket::{
    MessageLoopFuture, MultipleChannels, PeerId, PeerState, WebRtcChannel, WebRtcSocket,
    WebRtcSocketBuilder,
};
use tracing::{debug, error, trace, warn};

use crate::{
    message::{self, Clientbound, Delivery, Message, Serverbound, Symmetric},
    Flags,
};

pub struct SocketData {
    pub socket: WebRtcSocket<MultipleChannels>,
    pub reliable: WebRtcChannel,
    pub ordered: WebRtcChannel,
    pub unreliable: WebRtcChannel,
}

impl SocketData {
    pub fn broadcast(&mut self, msg: &impl Message) -> Result<()> {
        let delivery = Message::delivery(msg);
        let bytes = Message::serialize(msg)?;
        let packet = bytes.into_boxed_slice();
        let delivery = msg.delivery();

        // Send to all connected peers
        for peer in self.socket.connected_peers().collect::<Vec<_>>() {
            self.send_raw(peer, delivery, packet.clone())?;
        }

        Ok(())
    }

    pub fn send(&mut self, peer: PeerId, msg: &impl Message) -> Result<()> {
        let delivery = Message::delivery(msg);
        let bytes = Message::serialize(msg)?;
        let packet = bytes.into_boxed_slice();
        let delivery = msg.delivery();

        self.send_raw(peer, delivery, packet)
    }

    pub fn send_raw(&mut self, peer: PeerId, delivery: Delivery, packet: Box<[u8]>) -> Result<()> {
        trace!(?peer, ?delivery, len = packet.len(), "Send");

        let channel = match delivery {
            Delivery::Reliable => &mut self.reliable,
            Delivery::Ordered => &mut self.ordered,
            Delivery::Unreliable => &mut self.unreliable,
        };

        channel.send(packet, peer);

        Ok(())
    }

    /// Receive all pending messages from the network
    pub fn receive<T: Message>(&mut self) -> impl Iterator<Item = (PeerId, T)> {
        // Receive messages from all channels
        let packets = []
            .into_iter()
            .chain(
                self.reliable
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| debug!(?peer, ?packet, "Reliable")),
            )
            .chain(
                self.ordered
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| debug!(?peer, ?packet, "Ordered")),
            )
            .chain(
                self.unreliable
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| debug!(?peer, ?packet, "Unreliable")),
            );

        // Deserialize message data
        filter_errors(packets.map(|(p, b)| Message::deserialize(&b).map(|m| (p, m))))
    }
}




/// Get rid of Errs, logging them to stderr
fn filter_errors<T>(i: impl Iterator<Item = Result<T>>) -> impl Iterator<Item = T> {
    i.filter_map(|t| match t {
        Ok(value) => Some(value),
        Err(err) => {
            error!(?err, "Message error");
            None
        }
    })
}
