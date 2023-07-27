use anyhow::Result;
use instant::{Duration, Instant};
use tracing::{debug, trace};

use crate::{
    message::{Clientbound, Serverbound, Symmetric},
    Socket, StateData,
};

const PING_TIMEOUT: Duration = Duration::from_secs(1);

/// I am a client
pub struct ClientData {
    round_trip: Duration,
    last_ping: Instant,
    /// Serialized state received from the server in case of desync
    state: Vec<u8>,
}

/// Server has confirmed another tick
pub struct TickConfirm {
    /// Hash of the state after applying this tick's inputs
    hash: [u8; 8],
    /// Serialized state. If Some, the state should be deserialized before
    /// applying inputs, and then checked for hash
    state: Option<Vec<u8>>,
}

impl TickConfirm {
    /// Returns true if a state has diverged from teh server and sync is required
    pub fn diverged(&self, state: &impl StateData) -> bool {
        let host = self.hash;
        let client = state.default_hash();
        trace!(?host, ?client, "Checking diversion");
        host != client
    }

    /// Possibly deserialize
    pub fn state<S: StateData>(&self) -> Result<Option<S>> {
        if let Some(bytes) = &self.state {
            let state = S::default_deserialize(&bytes)?;
            let hash = state.default_hash();

            debug!(?hash, len = bytes.len(), "Deserializing");

            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}

impl ClientData {
    pub fn new() -> Self {
        Self {
            round_trip: Duration::ZERO,
            last_ping: Instant::now() - PING_TIMEOUT,
            state: vec![],
        }
    }

    pub fn poll(&mut self, socket: &mut Socket) -> Result<Option<TickConfirm>> {
        socket.poll();

        // Collected for lifetime reasons
        let Some((_, message)) = socket.receive::<Clientbound>()? else { return Ok(None) };

        let mut ret = None;

        match message {
            Clientbound::Symmetric(Symmetric::Ping) => {
                socket.broadcast(&Serverbound::Symmetric(Symmetric::Pong))?;
            }
            Clientbound::Symmetric(Symmetric::Pong) => {
                self.round_trip = self.last_ping.elapsed();
                self.last_ping += self.round_trip;
                // socket.broadcast(&Serverbound::Symmetric(Symmetric::Ping))?;

                trace!(rtt = ?self.round_trip, "Pong");
            }
            Clientbound::Tick { hash, tick_delta } => {
                trace!(host_hash = ?hash, ?tick_delta, "Tick");

                // Take the sync data if present
                let state = (!self.state.is_empty()).then(|| std::mem::take(&mut self.state));
                ret = Some(TickConfirm { hash, state });
            }
            Clientbound::SyncFragment { mut chunk } => {
                trace!(len = chunk.len(), "SyncFragment");
                self.state.append(&mut chunk)
            }
        }

        // Send pings every now and then
        if self.last_ping.elapsed() > PING_TIMEOUT {
            // Set last pong so timing will be accurate
            self.last_ping = Instant::now();
            socket.broadcast(&Serverbound::Symmetric(Symmetric::Ping))?;
        }

        Ok(ret)
    }

    /// Announce that the client has become desynced and resync is required
    pub fn desync(&mut self, socket: &mut Socket) -> Result<()> {
        socket.broadcast(&Serverbound::Sync)
    }
}
