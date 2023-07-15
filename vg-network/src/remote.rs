use std::future::Future;

use anyhow::{anyhow, Result};

use crate::socket::Socket;

pub struct Remote {
    socket: Socket,
}

impl Remote {
    pub fn new(url: &str) -> (Remote, impl Future<Output = Result<()>>) {
        let (socket, driver) = Socket::new(url);
        let driver = async { driver.await.map_err(|e| anyhow!("Socket error: {e}")) };

        (Remote { socket }, driver)
    }

    pub fn tick(&mut self) {
        self.socket.poll();
    }
}
