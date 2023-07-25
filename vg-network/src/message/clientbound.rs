use instant::Duration;
use serde::{Deserialize, Serialize};

use super::{Delivery, Message, Symmetric};

/// Messages sent by server, received by client
#[derive(Serialize, Deserialize)]
pub enum Clientbound {
    Symmetric(Symmetric),
    /// Server has performed a state tick
    Tick {
        hash: [u8; 8],
        tick_delta: Duration,
    },
    /// Fragment of a sync response
    SyncFragment {
        chunk: Vec<u8>,
    }
}

impl Message for Clientbound {
    fn delivery(&self) -> Delivery {
        match self {
            Clientbound::Symmetric(s) => s.delivery(),
            Clientbound::Tick { .. } => Delivery::Reliable,
            Clientbound::SyncFragment { .. } => Delivery::Reliable,
        }
    }
}
