use std::time::Duration;

use super::{recv, S2C};
use crate::{
    debug::Memory,
    net::{send, set_nodelay, C2S},
    runtime::Runtime,
};
use anyhow::Result;
use std::sync::mpsc::Sender as SyncSender;
use tokio::{select, sync::mpsc::Receiver};
use tokio_tungstenite::connect_async;
use tracing::*;
use vg_types::Event;

pub async fn run<RT: Runtime>(
    url: &str,
    mut rx: Receiver<Event>,
    tx: SyncSender<RT>,
) -> Result<()> {
    let (mut socket, _) = connect_async(url).await?;
    debug!("Connected to {}", url);

    // Make sending events and ticks faster
    set_nodelay(&mut socket);

    let (_player, mut runtime) = match recv(&mut socket).await? {
        Some(S2C::Sync { player, state }) => {
            debug!(
                "Received sync from server ({}), I am {:?}",
                Memory(state.len()),
                player
            );
            (player, RT::deserialize(&state)?)
        }
        other => {
            error!("Expected Sync, got {:?}", other);
            panic!();
        }
    };

    loop {
        select! {
            msg = recv::<S2C>(&mut socket) => {
                let msg = match msg {
                    Ok(Some(msg)) => msg,
                    Ok(None) => break,
                    Err(e) => return Err(e),
                };

                debug!("Received {:?}", msg);

                match msg {
                    S2C::Sync { .. } => todo!(),
                    S2C::Tick { events, tickrate_ns } => {
                        let tickrate = Duration::from_nanos(tickrate_ns);
                        for ev in events {
                            runtime.send(ev);
                        }
                        runtime.run_tick(tickrate)?;

                        let _ = tx.send(runtime.duplicate()?);
                    },
                }
            },
            Some(ev) = rx.recv() => {
                debug!("Sending event: {:?}", ev);
                send(&mut socket, C2S::Event(ev)).await?
            }
        }
    }

    info!("Disconnected from server");

    Ok(())
}
