use std::future::Future;

use anyhow::{anyhow, Result};
use vg_interface::WaitReason;
use vg_runtime::executor::Instance;

use crate::socket::Socket;

pub struct Local<I: Instance> {
    instance: I,
    socket: Socket,
}

impl<I: Instance> Local<I> {
    pub fn new(instance: I) -> (Self, impl Future<Output = Result<()>>) {
        let (socket, driver) = Socket::new("ws://localhost:3536");

        let driver = async {
            driver
                .await
                .map_err(|e| anyhow!("Socket driver error: {e}"))
        };

        (Self { instance, socket }, driver)
    }

    /// Lock events and execute tick
    pub fn tick(&mut self) {
        self.socket.poll();

        // Execute until tick is complete
        loop {
            match self.instance.step() {
                WaitReason::Startup => (),
                WaitReason::Present => break,
            }
        }
    }
}
