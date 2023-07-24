use anyhow::Result;
use matchbox_socket::ChannelConfig;
use serde::{de::DeserializeOwned, Serialize};

mod clientbound;
mod serverbound;

/// Delivery requirements for this kind of message
#[derive(Debug)]
pub enum Delivery {
    Reliable,
    Ordered,
    Unreliable,
}
impl Delivery {
    pub fn as_config(&self) -> ChannelConfig {
        match self {
            Delivery::Reliable => ChannelConfig {
                ordered: true,
                max_retransmits: None,
            },
            Delivery::Ordered => ChannelConfig {
                ordered: true,
                max_retransmits: Some(0),
            },
            Delivery::Unreliable => ChannelConfig {
                ordered: false,
                max_retransmits: Some(0),
            },
        }
    }
}

pub trait Message: Serialize + DeserializeOwned {
    fn delivery(&self) -> Delivery;

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}
