use serde::{Deserialize, Serialize};

use super::{Delivery, Message, Symmetric};

/// Messages sent by client, received by server
#[derive(Serialize, Deserialize)]
pub enum Serverbound {
    Symmetric(Symmetric),
}

impl Message for Serverbound {
    fn delivery(&self) -> Delivery {
        match self {
            Serverbound::Symmetric(m) => m.delivery(),
        }
    }
}
