use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::Message;
use dashmap::{mapref::one::RefMut, DashMap};
use matchbox_protocol::{JsonPeerEvent, PeerId};
use matchbox_signaling::SignalingState;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, error, info, trace, warn};

type AxumSender = UnboundedSender<Result<axum::extract::ws::Message, axum::Error>>;

#[derive(Clone)]
pub struct VgState {
    sockets: Arc<DashMap<SocketAddr, String>>,
    peers: Arc<DashMap<PeerId, String>>,
    rooms: Arc<DashMap<String, Room>>,
}

// #[derive(Clone)]
pub struct Room {
    key: String,
    host: Option<Connection>,
    clients: Vec<Connection>,
}

#[derive(Clone)]
pub struct Connection {
    peer: PeerId,
    sender: AxumSender,
}

impl Connection {
    pub fn id(&self) -> PeerId {
        self.peer
    }

    pub fn send(&self, msg: Message) {
        trace!(peer = ?self.peer, message = ?msg, "Message");
        if let Err(err) = self.sender.send(Ok(msg)) {
            error!(peer = ?self.peer, "Could not send message: {err}");
        }
    }
}

impl Room {
    fn new(key: String) -> Room {
        info!(room = key, "Created new room");
        Room {
            key,
            host: None,
            clients: vec![],
        }
    }

    /// Send a message to a recipient in the room
    pub fn send_to(&self, target: PeerId, msg: Message) {
        self.clients
            .iter()
            .chain(&self.host)
            .filter(|c| c.id() == target)
            .for_each(|c| c.send(msg.clone()))
    }

    /// Registers a new peer for this room
    ///
    /// Returns true if host got assigned
    pub fn add_peer(&mut self, peer: PeerId, sender: AxumSender) -> bool {
        let conn = Connection { peer, sender };

        if self.host.is_none() {
            debug!(room = ?self.key, peer = ?peer, "Assigned new host");
            self.host = Some(conn);
            true
        } else {
            debug!(room = ?self.key, peer = ?peer, "Added new client");
            self.clients.push(conn);
            false
        }
    }

    /// Remove a peer from this room, resetting it if host disconnected
    pub fn remove_peer(&mut self, peer: PeerId) {
        let Some(host) = self.host().cloned() else {
            return;
        };

        if host.id() == peer {
            debug!(room = ?self.key, peer = ?peer, "Host disconnected");

            // Host disconnected
            self.host = None;
            self.clients.clear();
        } else if let Some(idx) = self.clients.iter().position(|c| c.id() == peer) {
            debug!(room = ?self.key, peer = ?peer, "Client disconnected");

            // Notify peer of disconnection
            let client = self.clients.swap_remove(idx);
            host.send(Message::Text(
                JsonPeerEvent::PeerLeft(client.id()).to_string(),
            ));
        } else {
            warn!(room = ?self.key, peer = ?peer, "Peer tried to disconnect from room they are not in");
        }
    }

    /// Access the host for this room
    pub fn host(&self) -> Option<&Connection> {
        self.host.as_ref()
    }

    /// Get number of players currently in this room
    pub fn players(&self) -> usize {
        self.host.is_some() as usize + self.clients.len()
    }
}

impl VgState {
    pub fn new() -> Self {
        Self {
            sockets: Default::default(),
            peers: Default::default(),
            rooms: Default::default(),
        }
    }

    /// Register a request for one peer
    pub fn request(&self, addr: SocketAddr, key: String) {
        self.sockets.insert(addr, key);
    }

    /// Transfer the room key from a SocketAddr to a PeerId
    ///
    /// Returns false if address is not found
    pub fn upgrade(&self, addr: SocketAddr, peer: PeerId) -> bool {
        // Take the key from sockets and insert into peers
        let Some((_, key)) = self.sockets.remove(&addr) else {
            return false;
        };
        self.peers.insert(peer, key);
        true
    }

    pub fn key(&self, peer: PeerId) -> Option<String> {
        self.peers.get(&peer).map(|r| r.clone())
    }

    pub fn room_mut(&self, key: String) -> RefMut<'_, String, Room> {
        self.rooms
            .entry(key.clone())
            .or_insert_with(|| Room::new(key))
        // self.rooms.get_mut(key).map(|mut r| r.value_mut())
    }

    pub fn status(&self) -> Status {
        Status {
            version: SemVer {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            },
            rooms: self.rooms.len(),
            players: self.rooms.iter().fold(0, |s, r| s + r.players()),
        }
    }
}

#[derive(Serialize)]
pub struct SemVer {
    major: usize,
    minor: usize,
    patch: usize,
}

#[derive(Serialize)]
pub struct Status {
    version: SemVer,
    rooms: usize,
    players: usize,
}

impl SignalingState for VgState {}
