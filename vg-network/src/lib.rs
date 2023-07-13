use anyhow::Result;
use instant::Duration;
use local::Local;
use remote::Remote;
use vg_runtime::executor::*;

mod local;
mod messages;
mod remote;

pub enum Server<F, E: Executor<F> = DefaultExecutor<F>> {
    Local(Local<E::Instance>),
    Remote(Remote),
}

impl<F, E: Executor<F>> Server<F, E> {
    pub fn start(wasm: &[u8], debug: bool, func: F) -> Result<Self> {
        let instance = E::create(wasm, debug, func)?;
        Ok(Self::Local(Local::new(instance)))
    }

    pub fn connect() -> Result<Self> {
        Ok(Self::Remote(Remote::new()))
    }

    pub fn tick(&mut self) {
        match self {
            Server::Local(local) => local.tick(),
            Server::Remote(_) => todo!(),
        }
    }
}

pub struct ServerConfig {
    tickrate: Duration,
}

pub enum Event {}
