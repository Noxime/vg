use anyhow::Result;
use instant::Duration;
use tracing::{debug, trace, warn};

use crate::{
    message::{Clientbound, Serverbound, Symmetric},
    Socket, StateData,
};

/// I am a server
pub struct HostData {
    state: Vec<u8>,
}

impl HostData {
    pub fn new() -> HostData {
        HostData { state: vec![] }
    }

    pub fn poll(&mut self, socket: &mut Socket) -> Result<()> {
        socket.poll();

        // Collected for lifetime reasons
        let Some((peer, message)) = socket.receive::<Serverbound>()? else {
            return Ok(());
        };

        match message {
            Serverbound::Symmetric(Symmetric::Ping) => {
                trace!(?peer, "Ping");
                socket.broadcast(&Clientbound::Symmetric(Symmetric::Pong))?;
            }
            Serverbound::Symmetric(Symmetric::Pong) => {
                warn!(?peer, "Server received a pong (not supposed to)");
            }
            Serverbound::Sync => {
                let num_chunks = (self.state.len() + 1023) / 1024;
                debug!(
                    ?peer,
                    len = self.state.len(),
                    chunks = num_chunks,
                    "Synchronize"
                );

                // SCPT (WebRTC (Matchbox)) maximum packet size is 1280 something bytes
                for chunk in self.state.chunks(1024) {
                    socket.send(
                        peer,
                        &Clientbound::SyncFragment {
                            chunk: chunk.to_vec(),
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    // Register a new tick
    pub fn tick<S: StateData>(
        &mut self,
        socket: &mut Socket,
        state: &S,
        tick_delta: Duration,
    ) -> Result<()> {
        self.state = state.default_serialize()?;
        let hash: [u8; 8] = state.default_hash();

        {
            let check = S::default_deserialize(&self.state)?;
            assert_eq!(
                hash,
                check.default_hash(),
                "Serialized and deserialized hash mismatch"
            );
        }

        debug!(?hash, "Server tick");

        socket.broadcast(&Clientbound::Tick { hash, tick_delta })
    }
}
