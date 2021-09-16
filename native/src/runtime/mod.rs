use std::time::Duration;

use vg_types::Call;

// #[cfg(feature = "wasm")]
// pub mod interpreter;
// #[cfg(feature = "wasm")]
// pub mod wasmer;
#[cfg(feature = "wasm")]
pub mod wasmtime;

pub type Recommended = wasmtime::WasmtimeRT;
pub type Error = Box<dyn std::error::Error>;

pub trait Runtime
where
    Self: Sized,
{
    const NAME: &'static str;

    fn load(code: &[u8]) -> Result<Self, Error>;
    fn run_tick(&mut self, dt: Duration) -> Result<Vec<Call>, Error>;
    fn send(&mut self, value: vg_types::Response);

    fn serialize(&self) -> Result<Vec<u8>, Error>;
    fn deserialize(bytes: &[u8]) -> Result<Self, Error>;

    fn duplicate(&mut self) -> Result<Self, Error> {
        let bytes = self.serialize()?;
        Self::deserialize(&bytes)
    }
}
