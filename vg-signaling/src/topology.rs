use std::str::FromStr;

use anyhow::anyhow;
use async_trait::async_trait;
use axum::extract::ws::Message;
use futures_util::StreamExt;
use matchbox_protocol::{JsonPeerEvent, JsonPeerRequest, PeerRequest};
use matchbox_signaling::{NoCallbacks, SignalingTopology, WsStateMeta};
use tracing::{error, trace};

use crate::state::VgState;

pub struct VgTopology {}

impl VgTopology {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SignalingTopology<NoCallbacks, VgState> for VgTopology {
    // Called on WebSocket upgrade
    async fn state_machine(upgrade: WsStateMeta<NoCallbacks, VgState>) {
        let WsStateMeta {
            peer_id,
            sender,
            mut receiver,
            state,
            ..
        } = upgrade;

        let Some(key) = state.key(peer_id) else {
            error!(peer = ?peer_id, "Peer didn't have associated key");
            return
        };

        // Add new host or client
        {
            let mut room = state.room_mut(key.clone());
            let is_host = room.add_peer(peer_id, sender.clone());

            // Notify host of a new client connection
            if !is_host {
                let msg = Message::Text(JsonPeerEvent::NewPeer(peer_id).to_string());
                room.host().unwrap().send(msg);
            }
        }

        // As long as signaling connection is active
        while let Some(req) = receiver.next().await {
            let req = match parse_request(req) {
                Ok(req) => req,
                Err(err) => {
                    error!(peer = ?peer_id, "Failed to parse request: {err}");
                    break;
                }
            };

            match req {
                PeerRequest::Signal { receiver, data } => {
                    trace!(peer = ?peer_id, target = ?receiver, "Signaling");
                    let room = state.room_mut(key.clone());
                    room.send_to(receiver, Message::Text(data.to_string()));
                }
                PeerRequest::KeepAlive => {
                    trace!(peer = ?peer_id, "Keep alive");
                }
            }
        }

        let mut room = state.room_mut(key);
        room.remove_peer(peer_id);
    }
}

fn parse_request(req: Result<Message, axum::Error>) -> anyhow::Result<JsonPeerRequest> {
    match req? {
        Message::Text(text) => Ok(JsonPeerRequest::from_str(&text)?),
        Message::Close(_) => Err(anyhow!("Client disconnect")),
        m => Err(anyhow!("Unknown message type: {m:?}")),
    }
}
