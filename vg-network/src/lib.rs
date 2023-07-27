mod client;
mod flags;
mod host;
mod message;
mod socket;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use anyhow::Result;
pub use client::ClientData;
pub use flags::Flags;
pub use host::HostData;
use serde::{de::DeserializeOwned, Serialize};
pub use socket::{Role, Socket};

/// Shorthand for serializable and hashable state
pub trait StateData: Serialize + DeserializeOwned + Hash {
    fn default_hash(&self) -> [u8; 8] {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_le_bytes()
    }

    fn default_serialize(&self) -> Result<Vec<u8>> {
        compress(&bincode::serialize(self)?)
    }

    fn default_deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&decompress(bytes)?)?)
    }
}

impl<T: Serialize + DeserializeOwned + Hash> StateData for T {}

fn compress(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(bytes.to_vec())

    // let mut enc = snap::write::FrameEncoder::new(vec![]);
    // enc.write_all(bytes)?;

    // Ok(enc.into_inner()?)
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(bytes.to_vec())

    // let cursor = Cursor::new(bytes);
    // let mut buf = vec![];

    // let mut dec = snap::read::FrameDecoder::new(cursor);
    // dec.read_to_end(&mut buf)?;

    // Ok(buf)
}
