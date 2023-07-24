use serde::{Deserialize, Serialize};

use super::{Delivery, Message, Symmetric};

/// Messages sent by server, received by client
#[derive(Serialize, Deserialize)]
pub enum Clientbound {
    Symmetric(Symmetric),
}

impl Message for Clientbound {
    fn delivery(&self) -> Delivery {
        match self {
            Clientbound::Symmetric(s) => s.delivery(),
        }
    }
}
