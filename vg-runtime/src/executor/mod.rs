pub mod wasmtime;

use std::collections::HashMap;

use anyhow::Result;
use get_size::GetSize;
use vg_interface::WaitReason;

/// Executor recommended for this platform
pub type DefaultExecutor<F> = wasmtime::WasmtimeExecutor<F>;

/// WASM executor capable of producing Instances
pub trait Executor<F>: Sized {
    type Instance: Instance;

    fn create(wasm: &[u8], debug: bool, func: F) -> Result<Self::Instance>;
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
pub type PageData = [u8; PAGE_SIZE];

#[derive(GetSize)]
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
                .map(|page| page.try_into().expect("Non PAGE_SIZE aligned page"))
                .collect(),
        }
    }
}

#[derive(GetSize)]
pub enum GlobalData {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}

#[derive(GetSize)]
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

#[derive(GetSize)]
pub struct InstanceData {
    pub memories: HashMap<String, MemoryData>,
    pub globals: HashMap<String, GlobalData>,
    pub tables: HashMap<String, TableData>,
}
