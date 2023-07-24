use serde::{Deserialize, Serialize};

use super::{Delivery, Message};

#[derive(Serialize, Deserialize)]
pub enum Clientbound {
    Ping,
    Pong,
}

impl Message for Clientbound {
    fn delivery(&self) -> Delivery {
        match self {
            Clientbound::Ping => Delivery::Unreliable,
            Clientbound::Pong => Delivery::Unreliable,
        }
    }
}
