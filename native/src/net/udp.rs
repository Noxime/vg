use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    time::interval,
};
use tracing::*;

use crate::{debug::Memory, runtime::Runtime};

type Error = Box<dyn std::error::Error>;
const MAX_BYTES: usize = 1024;
const CHUNK_BYTES: usize = MAX_BYTES - 256;

pub struct Server<RT> {
    socket: UdpSocket,
    runtime: RT,
    clients: DashMap<SocketAddr, ServerState>,
}

#[derive(Copy, Clone)]
struct Shared<'a, RT> {
    socket: &'a UdpSocket,
    runtime: &'a RT,
}

enum ServerState {
    Sync { chunks: Vec<Chunk> },
    Game {},
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
enum C2S {
    Hello,
    AckChunk(usize),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
enum S2C {
    Chunk(Chunk),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chunk {
    i: usize,
    len: usize,
    bytes: Vec<u8>,
}

impl<RT: Runtime> Server<RT> {
    pub async fn bind(addr: impl ToSocketAddrs, code: Vec<u8>) -> Result<Self, Error> {
        let socket = UdpSocket::bind(addr).await?;
        debug!("Server on {}", socket.local_addr()?);

        let runtime = RT::load(&code)?;
        let clients = DashMap::new();

        Ok(Self {
            socket,
            runtime,
            clients,
        })
    }

    pub async fn run(mut self) {
        let tickrate = Duration::from_millis(100);
        let mut ticker = interval(tickrate);

        loop {
            // Listen for client messages until it is time to tick. It is safe to cancel UDP reads and intervals
            tokio::select!{
                _ = futures::future::join(self.listen(), self.process()) => {},
                _ = ticker.tick() => {}
            }
            
            self.run_tick(tickrate).unwrap();

            let shared = Shared {
                socket: &self.socket,
                runtime: &self.runtime,
            };

            for mut entry in self.clients.iter_mut() {
                entry.value_mut().run(shared);
            }
        }
    }

    fn run_tick(&mut self, tickrate: Duration) -> Result<(), Error> {
        let calls = self.runtime.run_tick(tickrate)?;
        trace!("Server tick generated {} calls", calls.len());

        Ok(())
    }

    async fn listen(&self) -> Result<(), Error> {
        while let Ok((msg, addr)) = self.recv().await {
            if let Some(c) = self.clients.get(&addr) {

            }
        }

        Ok(())
    }

    async fn process(&self) -> Result<(), Error> {
        for client in self.clients.iter() {
            let addr = *client.key();
            let state = client.value();

            match state {
                ServerState::Sync { chunks } => {
                    for chunk in chunks {
                        self.send(&S2C::Chunk(chunk.clone()), addr).await?;
                    }
                }
                ServerState::Game { .. } => {}
            }
        }

        Ok(())
    }

    async fn send(&self, msg: &S2C, addr: SocketAddr) -> Result<(), Error> {
        let bytes = bincode::serialize(msg)?;
        assert!(
            bytes.len() < MAX_BYTES,
            "Trying send too big of a message to client: {:#?}",
            msg,
        );
        assert_eq!(
            self.socket.send_to(&bytes, addr).await?,
            bytes.len(),
            "Sent message to server but got fragmented, we should lower MAX_BYTES"
        );
        Ok(())
    }

    async fn recv(&self) -> Result<(C2S, SocketAddr), Error> {
        let mut buf = [0; MAX_BYTES];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        trace!("Received {} bytes from {}", len, addr);

        let msg = bincode::deserialize(&buf[..len])?;
        trace!("Deserialized message from {}: {:#?}", addr, msg);

        Ok((msg, addr))
    }
}

impl ServerState {
    async fn run<'a, RT>(&'a mut self, shared: Shared<'a, RT>) {
        while let Some(msg) = channel.rx.recv().await {
            match msg {
                C2S::Hello => {
                    let bytes = self.runtime.serialize()?;
                    // Divide, rounding up
                    let num_chunks = (bytes.len() + CHUNK_BYTES - 1) / CHUNK_BYTES;

                    debug!(
                        "Sending player {} bytes state, in {} chunks",
                        Memory(bytes.len()),
                        num_chunks
                    );

                    let chunks = bytes
                        .chunks(CHUNK_BYTES)
                        .enumerate()
                        .map(|(i, chunk)| {
                            trace!("Sending chunk {} of {}", i, num_chunks);
                            Chunk {
                                i,
                                len: num_chunks,
                                bytes: chunk.to_vec(),
                            }
                        })
                        .collect();

                    self.clients.insert(addr, ServerState::Sync { chunks });
                }
                C2S::AckChunk(i) => {
                    if let Some(mut r) = self.clients.get_mut(&addr) {
                        let mut done = false;

                        if let ServerState::Sync { chunks } = r.value_mut() {
                            chunks.retain(|c| c.i != i);
                            if chunks.is_empty() {
                                done = true;
                            }
                        }

                        if done {
                            debug!("Server sync complete");
                            *r = ServerState::Game {};
                        }
                    }
                }
            }
        }
    }
}

pub struct Client<RT> {
    socket: UdpSocket,
    state: ClientState<RT>,
    rtt: Duration,
}

enum ClientState<RT> {
    Sync { chunks: Vec<Chunk> },
    Game { runtime: RT },
}

impl<RT: Runtime> Client<RT> {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        debug!("Connected! I am {}", socket.local_addr()?);

        let state = ClientState::Sync { chunks: vec![] };
        let c = Self {
            socket,
            state,
            rtt: Duration::ZERO,
        };

        // Let them know we are connected!
        c.send(&C2S::Hello).await?;

        Ok(c)
    }

    pub async fn run(mut self) -> Result<(), Error> {
        while let Ok(msg) = self.recv().await {
            match msg {
                S2C::Chunk(chunk) => {
                    trace!("Received chunk {}/{} from server", chunk.i, chunk.len);
                    self.send(&C2S::AckChunk(chunk.i)).await?;

                    let mut done = false;
                    if let ClientState::Sync { ref mut chunks } = self.state {
                        let len = chunk.len;
                        chunks.push(chunk);

                        // Maybe not do this constantly
                        chunks.sort_by_key(|c| c.i);
                        chunks.dedup_by_key(|c| c.i);

                        // We remove duplicates, so an incomplete list is always shorter than the full list
                        if chunks.len() == len {
                            debug!("Client sync complete");
                            done = true;
                        }

                        if done {
                            let bytes: Vec<u8> = chunks
                                .into_iter()
                                .map(|c| c.bytes.iter())
                                .flatten()
                                .copied()
                                .collect();

                            let runtime = RT::deserialize(&bytes)?;
                            self.state = ClientState::Game { runtime };
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn send(&self, msg: &C2S) -> Result<(), Error> {
        trace!("Sending {:#?} to server", msg);

        let bytes = bincode::serialize(&msg)?;
        assert!(
            bytes.len() < MAX_BYTES,
            "Trying send too big of a message to server: {:#?}",
            msg,
        );
        assert_eq!(
            self.socket.send(&bytes).await?,
            bytes.len(),
            "Sent message to server but got fragmented, we should lower MAX_BYTES"
        );
        Ok(())
    }

    async fn recv(&self) -> Result<S2C, Error> {
        let mut buf = [0; MAX_BYTES];
        let len = self.socket.recv(&mut buf).await?;
        trace!("Received {} bytes from server", len);

        let msg = bincode::deserialize(&buf[..len])?;
        trace!("Deserialized message from server: {:#?}", msg);

        Ok(msg)
    }
}
