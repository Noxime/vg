use anyhow::Result;
use std::time::Duration;

use vg_types::Call;

// #[cfg(feature = "wasm")]
// pub mod interpreter;
// #[cfg(feature = "wasm")]
// pub mod wasmer;
#[cfg(feature = "wasm")]
pub mod wasmtime;

pub type Recommended = wasmtime::WasmtimeRT;

pub trait Runtime
where
    Self: Sized + Send,
{
    const NAME: &'static str;

    fn load(code: &[u8]) -> Result<Self>;
    fn run_tick(&mut self, dt: Duration) -> Result<Vec<Call>>;
    fn send(&mut self, value: vg_types::PlayerEvent);

    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(bytes: &[u8]) -> Result<Self>;

    fn duplicate(&mut self) -> Result<Self> {
        let bytes = self.serialize()?;
        Self::deserialize(&bytes)
    }
}
