use std::collections::VecDeque;

use anyhow::Result;

use matchbox_socket::{
    MessageLoopFuture, MultipleChannels, Packet, PeerId, PeerState, WebRtcChannel, WebRtcSocket,
    WebRtcSocketBuilder,
};
use tracing::{debug, trace};

use crate::{
    message::{Delivery, Message},
    Flags,
};

pub struct Socket {
    pub socket: WebRtcSocket<MultipleChannels>,
    pub reliable: WebRtcChannel,
    pub ordered: WebRtcChannel,
    pub unreliable: WebRtcChannel,
    pub queue: VecDeque<(PeerId, Packet)>,
    pub role: Option<Role>,
    pub stats: SocketStats,
}

/// The assigned room role for this socket
#[derive(Clone, Copy, Debug)]
pub enum Role {
    /// This socket is a host (authority)
    Host,
    /// This socket is a client
    Client,
}

#[derive(Debug, Clone, Copy)]
pub struct SocketStats {
    sent_bytes: usize,
    sent_packets: usize,
    recv_bytes: usize,
    recv_packets: usize,
}

impl SocketStats {
    pub fn total_bytes(&self) -> usize {
        self.recv_bytes + self.sent_bytes
    }

    pub fn total_packets(&self) -> usize {
        self.recv_packets + self.sent_packets
    }

    pub fn average_bytes(&self) -> usize {
        self.total_bytes() / self.total_packets()
    }
}

impl Socket {
    pub fn new(url: &str) -> Result<(Socket, MessageLoopFuture)> {
        let (mut socket, driver) = WebRtcSocketBuilder::new(url)
            .reconnect_attempts(None)
            .add_channel(Delivery::Reliable.as_config())
            .add_channel(Delivery::Ordered.as_config())
            .add_channel(Delivery::Unreliable.as_config())
            .build();

        Ok((
            Socket {
                reliable: socket.take_channel(0).unwrap(),
                ordered: socket.take_channel(1).unwrap(),
                unreliable: socket.take_channel(2).unwrap(),
                socket,
                queue: Default::default(),
                role: None,
                stats: SocketStats {
                    sent_bytes: 0,
                    sent_packets: 0,
                    recv_bytes: 0,
                    recv_packets: 0,
                },
            },
            driver,
        ))
    }

    pub fn poll(&mut self) {
        for (peer, state) in self.socket.update_peers() {
            debug!(?peer, ?state, "Peer state change");
            self.update_flags(peer, state);
        }
    }

    /// Poll the socket, waiting for role assignment
    pub fn poll_role(&mut self) -> Option<Role> {
        self.poll();
        self.role
    }

    /// Looks for magic flag events
    fn update_flags(&mut self, peer: PeerId, state: PeerState) {
        if state != PeerState::Disconnected {
            return;
        }

        // Short out if not actual peer data (magic mismatch etc)
        let Ok(flags) = Flags::decode(peer) else {
            return;
        };

        debug!(?flags, "Received flags");

        // We are not the room host
        self.role = Some(if flags.is_host {
            Role::Host
        } else {
            Role::Client
        });
    }

    pub fn broadcast(&mut self, msg: &impl Message) -> Result<()> {
        let _delivery = Message::delivery(msg);
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
        let _delivery = Message::delivery(msg);
        let bytes = Message::serialize(msg)?;
        let packet = bytes.into_boxed_slice();
        let delivery = msg.delivery();

        self.send_raw(peer, delivery, packet)
    }

    fn send_raw(&mut self, peer: PeerId, delivery: Delivery, packet: Box<[u8]>) -> Result<()> {
        trace!(?peer, ?delivery, len = packet.len(), "Send");

        self.stats.sent_packets += 1;
        self.stats.sent_bytes += packet.len();

        let channel = match delivery {
            Delivery::Reliable => &mut self.reliable,
            Delivery::Ordered => &mut self.ordered,
            Delivery::Unreliable => &mut self.unreliable,
        };

        channel.send(packet, peer);

        Ok(())
    }

    /// Receive all pending messages from the network
    pub fn receive<T: Message>(&mut self) -> Result<Option<(PeerId, T)>> {
        // Receive messages from all channels
        let packets = []
            .into_iter()
            .chain(
                self.reliable
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| trace!(?peer, ?packet, "Reliable")),
            )
            .chain(
                self.ordered
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| trace!(?peer, ?packet, "Ordered")),
            )
            .chain(
                self.unreliable
                    .receive()
                    .into_iter()
                    .inspect(|(peer, packet)| trace!(?peer, ?packet, "Unreliable")),
            );

        // Add all new messages to the queue
        self.queue.extend(packets);

        // Deserialize message data
        let Some((peer, packet)) = self.queue.pop_front() else {
            return Ok(None);
        };

        self.stats.recv_packets += 1;
        self.stats.recv_bytes += packet.len();

        let message = Message::deserialize(&packet)?;
        Ok(Some((peer, message)))
    }

    pub fn stats(&self) -> SocketStats {
        self.stats
    }
}
