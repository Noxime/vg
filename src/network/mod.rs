use std::{
    sync::{atomic::Ordering::Relaxed, Arc},
    time::Duration,
};

use futures::{channel::mpsc::SendError, join};
use log::*;
// use lz4_flex::block::DecompressError;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;

use crate::{Event, Game, PlayerEvent, PlayerId, Vg};

pub(crate) struct NetPlayer {
    conn: Conn,
    id: PlayerId,
}

mod connection;
pub(crate) use connection::{Conn, Listener};
mod server;
use server::Server;
mod client;
use client::Client;

pub(crate) struct Network<G: Game> {
    pub server: Option<Server<G>>,
    pub client: Client<G>,
}

impl<G: Game> Network<G> {
    pub(crate) async fn new(state: Vg<G>) -> Result<Self, NetError> {
        let is_ready = Arc::new(AtomicBool::new(false));

        let server = Server::new(state).await.ok();

        let server_block = async {
            // Our server might not start if there is already a server running on this machine. That's okay
            // and intended while Vg is in developing state
            if let Some(mut server) = server {
                while !is_ready.load(Relaxed) {
                    server.poll().await.unwrap();
                }
                Some(server)
            } else {
                None
            }
        };

        let client_block = async {
            info!("Connecting");
            let client = Client::new().await;
            info!("Client ready");
            is_ready.store(true, Relaxed);
            client
        };

        let (client, server) = join!(client_block, server_block);
        // let (server, client) = join!(server_block, client_block);

        Ok(Self {
            server,
            client: client?,
        })
    }

    pub async fn poll(&mut self) -> Result<Option<NetEvent<G>>, NetError> {
        let mut server = self.server.take();

        let server_block = async {
            if let Some(mut s) = server.take() {
                s.poll().await?;
                Ok::<_, NetError>(Some(s))
            } else {
                Ok(server)
            }
        };

        let client_block = async { self.client.poll().await };

        let (s, c) = join!(server_block, client_block);

        self.server = s?;

        c
    }
}

pub(crate) enum NetEvent<G: Game> {
    /// A new server tick has occurred, this is the new state
    Tick(Vg<G>, Duration),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum S2C {
    Sync {
        state: Vec<u8>,
        id: PlayerId,
    },
    Tick {
        events: Vec<(Duration, PlayerEvent)>,
        delta: Duration,
        rollback: Duration,
        hash: u64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum C2S {
    Event { event: Event },
    Desync,
}

#[derive(Debug)]
pub(crate) enum NetError {
    OneRecv(tokio::sync::oneshot::error::RecvError),
    MpscRecv(tokio::sync::mpsc::error::RecvError),
    MpscSend(tokio::sync::mpsc::error::SendError<Vec<u8>>),
    Net(tungstenite::Error),
    Laminar(laminar::ErrorKind),
    Serde(bincode::Error),
    // Lz4(DecompressError),
}

impl From<tokio::sync::oneshot::error::RecvError> for NetError {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
        NetError::OneRecv(e)
    }
}

impl From<tokio::sync::mpsc::error::RecvError> for NetError {
    fn from(e: tokio::sync::mpsc::error::RecvError) -> Self {
        NetError::MpscRecv(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Vec<u8>>> for NetError {
    fn from(e: tokio::sync::mpsc::error::SendError<Vec<u8>>) -> Self {
        NetError::MpscSend(e)
    }
}

impl From<tungstenite::Error> for NetError {
    fn from(e: tungstenite::Error) -> Self {
        NetError::Net(e)
    }
}

impl From<laminar::ErrorKind> for NetError {
    fn from(e: laminar::ErrorKind) -> Self {
        NetError::Laminar(e)
    }
}

impl From<std::io::Error> for NetError {
    fn from(e: std::io::Error) -> Self {
        NetError::Net(tungstenite::Error::Io(e))
    }
}

impl From<bincode::Error> for NetError {
    fn from(e: bincode::Error) -> Self {
        NetError::Serde(e)
    }
}

// impl From<DecompressError> for NetError {
//     fn from(e: DecompressError) -> Self {
//         NetError::Lz4(e)
//     }
// }
