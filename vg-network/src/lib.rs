mod client;
mod flags;
mod message;
mod server;
mod socket;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub use flags::Flags;
use serde::{de::DeserializeOwned, Serialize};
pub use socket::{Socket, Role};
pub use client::ClientData;
pub use server::HostData;

/// Shorthand for serializable and hashable state
pub trait StateData: Serialize + DeserializeOwned + Hash {
    fn default_hash(&self) -> [u8; 8] {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_le_bytes()
    }
}

impl<T: Serialize + DeserializeOwned + Hash> StateData for T {}
