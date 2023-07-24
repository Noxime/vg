use serde::{Deserialize, Serialize};

use super::{Delivery, Message};

/// Message type that can be sent in any direction. However, that does not mean
/// behavior has to be symmetric. But it often is :)
#[derive(Serialize, Deserialize)]
pub enum Symmetric {
    Ping,
    Pong,
}

impl Message for Symmetric {
    fn delivery(&self) -> Delivery {
        match self {
            Symmetric::Ping => Delivery::Ordered,
            Symmetric::Pong => Delivery::Ordered,
        }
    }
}
