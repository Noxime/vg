use serde::{Deserialize, Serialize};

use super::{Delivery, Message};

#[derive(Serialize, Deserialize)]
pub enum Serverbound {
    Ping,
    Pong,
}

impl Message for Serverbound {
    fn delivery(&self) -> Delivery {
        match self {
            Serverbound::Ping => Delivery::Unreliable,
            Serverbound::Pong => Delivery::Unreliable,
        }
    }
}
