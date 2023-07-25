pub mod wasmtime;

use std::collections::BTreeMap;

use anyhow::Result;
use get_size::GetSize;
use serde::{de::Visitor, Deserialize, Serialize};
use vg_interface::{Request, Response, WaitReason};

/// Executor recommended for this platform
pub type DefaultExecutor = wasmtime::WasmtimeExecutor;

/// WASM executor capable of producing Instances
pub trait Executor: Sized {
    type Instance: Instance;

    fn create(
        wasm: &[u8],
        debug: bool,
        func: impl FnMut(Request) -> Response + 'static,
    ) -> Result<Self::Instance>;
}

/// Instance of a WebAssembly module that can be de/serialized
pub trait Instance {
    /// Step instance state by one. Note that this is different from a _tick_
    fn step(&mut self) -> WaitReason;
    /// Serialize instance data
    fn get_data(&mut self) -> InstanceData;
    /// Deserialize in place. Data must come from identical Instance
    fn set_data(&mut self, data: &InstanceData);
}

pub const PAGE_SIZE: usize = 65_536;

#[derive(GetSize, Hash)]
// #[derive(GetSize, Serialize, Deserialize, Hash)]
pub struct PageData {
    // TODO: Get rid of this allocation
    // There was an issue with serde overflowing the stack decoding 64k pages
    // #[serde(with = "serde_arrays")]
    bytes: [u8; PAGE_SIZE],
}

impl Serialize for PageData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.bytes)
    }
}

struct PageVisitor;
impl Visitor<'_> for PageVisitor {
    type Value = PageData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expected {PAGE_SIZE} bytes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut bytes = [0; PAGE_SIZE];
        bytes.copy_from_slice(v);
        Ok(PageData { bytes })
    }
}

impl<'de> Deserialize<'de> for PageData {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PageVisitor)
    }
}
//*/
#[derive(GetSize, Serialize, Deserialize, Hash)]
pub struct MemoryData {
    pub pages: Vec<PageData>,
}

impl MemoryData {
    /// Gets number of pages in this memory
    pub fn pages(&self) -> u64 {
        self.pages.len() as u64
    }
}

impl MemoryData {
    /// Bytes must be a multiple of PAGE_SIZE
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            pages: bytes
                .chunks(PAGE_SIZE)
                .map(|page| PageData {
                    bytes: page.try_into().expect("Must be aligned to PAGE_SIZE"),
                })
                .collect(),
        }
    }
}

#[derive(GetSize, Serialize, Deserialize, Hash)]
pub enum GlobalData {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}

#[derive(GetSize, Serialize, Deserialize, Hash)]
pub enum TableData {
    I32(Vec<i32>),
    I64(Vec<i64>),
    F32(Vec<u32>),
    F64(Vec<u64>),
}

impl TableData {
    /// Number of items in this table
    pub fn len(&self) -> u32 {
        match self {
            TableData::I32(v) => v.len() as u32,
            TableData::I64(v) => v.len() as u32,
            TableData::F32(v) => v.len() as u32,
            TableData::F64(v) => v.len() as u32,
        }
    }
}

#[derive(GetSize, Serialize, Deserialize, Hash)]
pub struct InstanceData {
    pub memories: BTreeMap<String, MemoryData>,
    pub globals: BTreeMap<String, GlobalData>,
    pub tables: BTreeMap<String, TableData>,
}
