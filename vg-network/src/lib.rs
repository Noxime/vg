use std::future::Future;

use anyhow::{anyhow, Result};
use socket::Socket;
use tracing::debug;
use vg_interface::{Request, Response, WaitReason};
use vg_runtime::executor::*;

mod socket;

pub struct Host<E: Executor = DefaultExecutor> {
    instance: E::Instance,
    socket: Socket,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub debug: bool,
    pub url: String,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            debug: true,
            url: "ws://vg.noxim.xyz:3536".into(),
        }
    }
}

impl<E: Executor> Host<E> {
    pub fn new(
        wasm: &[u8],
        func: impl FnMut(Request) -> Response + 'static,
        config: HostConfig,
    ) -> Result<(Self, impl Future<Output = Result<()>>)> {
        debug!(?config, "Creating host");

        let instance = E::create(wasm, config.debug, func)?;
        let (socket, driver) = Socket::new(&config.url);

        let driver = async { driver.await.map_err(|err| anyhow!("Socket error: {err}")) };

        Ok((Host { instance, socket }, driver))
    }

    pub fn tick(&mut self) {
        self.socket.poll();

        loop {
            match self.instance.step() {
                WaitReason::Startup => (),
                WaitReason::Present => break,
            }
        }
    }
}
