use log::*;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite as tg;
use tokio_tungstenite::tungstenite;
use tungstenite::Message;

use futures::{SinkExt, StreamExt};

use std::time::{Duration, Instant};

use super::{Event, EventKind, Game, PlayerId, Vg};

type Ws = tg::WebSocketStream<TcpStream>;

// client to server
#[derive(Debug, Serialize, Deserialize)]
enum MessageC2S {
    /// Clint detected a desync and wants to receive the full game state
    Desync(u64),
    // New client events
    Event(Vec<EventKind>),
}

#[derive(Debug, Serialize, Deserialize)]
enum MessageS2C {
    // Server ran an update tick, with hash and accepted events
    Tick(u64, Vec<Event>),
    // Force client into given state, usually because of C2S::Desync
    SetState { state: Vec<u8> },
}

pub(crate) struct Peer {
    ws: Ws,
    id: PlayerId,
    events: Vec<EventKind>,
}

impl Peer {
    fn new(ws: Ws) -> Peer {
        Peer {
            id: PlayerId::from(ws.get_ref().peer_addr().unwrap()),
            ws,
            events: vec![EventKind::Connected],
        }
    }
}

pub(crate) enum Network {
    Host {
        socket: TcpListener,
        peers: Vec<Peer>,
        last_tick: Instant,
    },
    Client {
        remote: Ws,
        ping_send: Option<Instant>,
        round_trip: Duration,
    },
}

async fn handshake(stream: TcpStream, addr: std::net::SocketAddr) -> Result<Ws, NetError> {
    trace!("TCP connection from '{}'", addr);
    let ws = tg::accept_async(stream).await?;
    info!("Shook hands with '{}'", addr);
    Ok(ws)
}

fn hash_state(g: &impl Game) -> u64 {
    fxhash::hash(&serde_json::to_vec(g).unwrap_or_default()) as u64
}

impl Network {
    pub(crate) async fn new() -> Result<Self, NetError> {
        // Are we host or client
        if let Ok(addr) = std::env::var("VG_CONNECT") {
            let (remote, _) = tg::connect_async(&addr).await?;
            debug!("Connected to {}", addr);

            Ok(Self::Client {
                remote,
                ping_send: None,
                round_trip: Default::default(),
            })
        } else {
            // port 0 will allocate an unused port on this machine
            let server_addr = std::env::var("VG_BIND").unwrap_or("localhost:0".into());
            let socket = TcpListener::bind(server_addr).await?;
            let server_addr = socket.local_addr()?;
            debug!("Server running on {}", server_addr);

            Ok(Self::Host {
                socket,
                peers: vec![],
                last_tick: Instant::now(),
            })
        }
    }

    pub(crate) fn is_host(&self) -> bool {
        if let Network::Host { .. } = self {
            true
        } else {
            false
        }
    }
}

impl<G: Game> Vg<G> {
    /// Accept any new pending connections or perform client bookkeeping
    // TODO: Break all of this up
    pub(crate) async fn update_network(&mut self) -> Result<bool, NetError> {
        let i_am = self.i_am();

        match &mut self.network {
            Network::Host {
                socket,
                peers,
                last_tick,
                ..
            } => {
                // Accept any new connections that might be pending
                match timeout(Duration::default(), socket.accept()).await {
                    Ok(Ok((stream, addr))) => {
                        let mut ws = handshake(stream, addr).await?;
                        let state = serde_json::to_vec(&self.state)?;
                        ws.send(Message::Binary(serde_json::to_vec(
                            &MessageS2C::SetState { state },
                        )?))
                        .await?;
                        peers.push(Peer::new(ws));
                    }
                    _ => (), // timeout or connection error
                }

                // Grab the websockets
                for mut peer in std::mem::replace(peers, vec![]) {
                    let addr = peer.ws.get_ref().peer_addr()?;
                    while let Ok(Some(msg)) = timeout(Duration::default(), peer.ws.next()).await {
                        match msg? {
                            Message::Binary(msg) => match serde_json::from_slice(&msg)? {
                                MessageC2S::Event(mut client_events) => {
                                    trace!("Client '{}' events: {:?}", addr, client_events);
                                    peer.events.append(&mut client_events);
                                }
                                MessageC2S::Desync(their_hash) => {
                                    debug!(
                                        "Client '{}' got desynced, their hash: {:016X}",
                                        addr, their_hash
                                    );
                                    let state = serde_json::to_vec(&self.state)?;
                                    peer.ws
                                        .send(Message::Binary(serde_json::to_vec(
                                            &MessageS2C::SetState { state },
                                        )?))
                                        .await?;
                                }
                            },
                            msg => {
                                trace!("Client '{}' message ignored: {:?}", addr, msg);
                            }
                        }
                    }

                    // Add live websockets back to peers list, so drop faulting ones
                    peers.push(peer);
                }

                // It's time for a tick
                if last_tick.elapsed() >= Duration::from_secs(1) / self.options.tick_rate as u32 {
                    debug!("Tick! ({:.2} tps)", 1.0 / last_tick.elapsed().as_secs_f32());
                    *last_tick = Instant::now();

                    // add all peer events
                    let mut all_events = vec![];
                    for peer in peers.iter_mut() {
                        for kind in std::mem::replace(&mut peer.events, vec![]) {
                            all_events.push(Event {
                                player: peer.id,
                                kind,
                            })
                        }
                    }
                    // add our local events to the queue
                    for kind in std::mem::replace(&mut self.event_queue, vec![]) {
                        all_events.push(Event { player: i_am, kind });
                    }

                    trace!("Tick events: {}", all_events.len());

                    let hash = hash_state(&self.state);

                    // Broadcast the tick to all clients
                    let msg = serde_json::to_vec(&MessageS2C::Tick(hash, all_events.clone()))?;
                    futures::future::try_join_all(
                        peers
                            .iter_mut()
                            .map(|p| p.ws.send(Message::binary(msg.clone()))),
                    )
                    .await?;

                    self.events = all_events;
                    return Ok(true); // run game tick
                }
            }
            Network::Client {
                remote,
                ping_send,
                round_trip,
            } => {
                // There is no ping message in flight, send one
                if ping_send.is_none() {
                    *ping_send = Some(Instant::now());
                    remote.send(Message::Ping(vec![])).await?;
                }

                // send any queued events to the server
                if !self.event_queue.is_empty() {
                    remote
                        .send(Message::binary(
                            serde_json::to_vec(&MessageC2S::Event(std::mem::replace(
                                &mut self.event_queue,
                                vec![],
                            )))
                            .unwrap(),
                        ))
                        .await?;
                }

                // First OK is timeout, Some means WS connection is OK
                while let Ok(Some(msg)) = timeout(Duration::default(), remote.next()).await {
                    match msg? {
                        Message::Pong(_) => {
                            if ping_send.is_some() {
                                *round_trip = ping_send.take().unwrap().elapsed();
                                debug!("Roundtrip time: {:.2?}", round_trip);
                            }
                        }
                        Message::Binary(msg) => match serde_json::from_slice(&msg)? {
                            MessageS2C::Tick(hash, events) => {
                                // Make sure we are still synchronized
                                let my_hash = hash_state(&self.state);
                                if my_hash != hash {
                                    warn!(
                                        "Desync! Server/Local: {:016X} != {:016X}",
                                        hash, my_hash
                                    );
                                    remote
                                        .send(Message::binary(serde_json::to_vec(
                                            &MessageC2S::Desync(hash),
                                        )?))
                                        .await?;
                                }

                                debug!("Tick! ({} events)", events.len());
                                self.events = events;
                                return Ok(true); // run game tick
                            }
                            MessageS2C::SetState { state } => {
                                debug!("Synced state from server ({} bytes)", state.len());
                                self.state = serde_json::from_slice(&state)?;
                            }
                        },
                        msg => {
                            trace!("Server message ignored: {:?}", msg);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get the last ping to the server
    pub fn latency(&self) -> Duration {
        self.round_trip() / 2 // Back and forth. This is the best approx we have
    }

    /// Get the round trip time of messages from this client to the server, and back. If a local game, zero
    pub fn round_trip(&self) -> Duration {
        if let Network::Client { round_trip, .. } = self.network {
            round_trip
        } else {
            Duration::from_secs(0)
        }
    }
}

#[derive(Debug)]
pub(crate) enum NetError {
    Net(tungstenite::Error),
    Json(serde_json::Error),
}

impl From<tungstenite::Error> for NetError {
    fn from(e: tungstenite::Error) -> Self {
        NetError::Net(e)
    }
}

impl From<std::io::Error> for NetError {
    fn from(e: std::io::Error) -> Self {
        NetError::Net(tungstenite::Error::Io(e))
    }
}

impl From<serde_json::Error> for NetError {
    fn from(e: serde_json::Error) -> Self {
        NetError::Json(e)
    }
}
