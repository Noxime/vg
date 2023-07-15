use std::future::Future;

use anyhow::Result;
use local::Local;
use remote::Remote;
use vg_interface::{Request, Response};
use vg_runtime::executor::*;

mod local;
mod remote;
mod socket;

pub enum Host<E: Executor = DefaultExecutor> {
    Local(Local<E::Instance>),
    Remote(Remote),
}

impl<E: Executor> Host<E> {
    pub fn start(
        wasm: &[u8],
        debug: bool,
        func: impl FnMut(Request) -> Response + 'static,
    ) -> Result<(Self, impl Future<Output = Result<()>>)> {
        let instance = E::create(wasm, debug, func)?;
        let (local, driver) = Local::new(instance);
        Ok((Self::Local(local), driver))
    }

    pub fn connect(url: &str) -> Result<(Self, impl Future<Output = Result<()>>)> {
        let (remote, driver) = Remote::new(url);
        Ok((Self::Remote(remote), driver))
    }

    pub fn tick(&mut self) {
        match self {
            Host::Local(local) => local.tick(),
            Host::Remote(remote) => remote.tick(),
        }
    }
}
