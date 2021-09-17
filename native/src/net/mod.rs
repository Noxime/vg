mod client;
mod server;

use anyhow::Result;
pub use client::run as client;
use derivative::Derivative;
use futures::{SinkExt, StreamExt};
pub use server::run as server;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{trace, warn};
use vg_types::{DeBin, SerBin};
use vg_types::{Event, PlayerEvent, PlayerId};

use std::fmt::Debug;

pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(SerBin, DeBin, Derivative)]
#[derivative(Debug)]
pub enum S2C {
    Sync {
        player: PlayerId,
        #[derivative(Debug = "ignore")]
        state: Vec<u8>,
    },
    Tick {
        events: Vec<PlayerEvent>,
        tickrate_ns: u64,
    },
}

#[derive(SerBin, DeBin, Derivative)]
#[derivative(Debug)]
pub enum C2S {
    Event(Event),
}

pub async fn send<T: SerBin + Debug>(socket: &mut Socket, msg: T) -> Result<()> {
    let bytes = msg.serialize_bin();
    trace!("Sending {:?} ({} bytes)", msg, bytes.len());
    socket.send(Message::binary(bytes)).await?;
    Ok(())
}

pub async fn recv<T: DeBin + Debug>(socket: &mut Socket) -> Result<Option<T>> {
    while let Some(res) = socket.next().await {
        let msg = res?;

        match msg {
            Message::Binary(vec) => {
                let msg = T::deserialize_bin(&vec)?;
                trace!("Received {:?} ({})", msg, vec.len());
                return Ok(Some(msg));
            }
            _ => {
                warn!("Received unhandled message: {:?}", msg);
                continue;
            }
        }
    }

    Ok(None)
}

fn set_nodelay(socket: &mut Socket) {
    if let MaybeTlsStream::Plain(s) = socket.get_ref() {
        s.set_nodelay(true).unwrap();
    }
}
