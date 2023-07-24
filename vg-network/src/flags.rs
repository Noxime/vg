use anyhow::{Result, anyhow};
use matchbox_socket::PeerId;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct Flags {
    pub is_host: bool,
}

impl Flags {
    const MAGIC: [u8; 8] = *b"CAFEBABE";

    pub fn encode(&self) -> PeerId {
        let magic = u64::from_le_bytes(Self::MAGIC);
        let flags = (self.is_host as u64) << 0;

        PeerId(Uuid::from_u64_pair(magic, flags))
    }

    pub fn decode(peer: PeerId) -> Result<Self> {
        let (magic, flags) = peer.0.as_u64_pair();

        if magic.to_le_bytes() != Self::MAGIC {
            return Err(anyhow!("Mismatched error: {magic} != {:?}", Self::MAGIC));
        }

        Ok(Self {
            is_host: (flags << 0) & 1 != 0,
        })
    }
}