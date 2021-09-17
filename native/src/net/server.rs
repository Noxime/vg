use super::Socket;
use crate::{
    debug::Memory,
    ids::IdSource,
    net::{recv, send, set_nodelay, C2S, S2C},
    runtime::Runtime,
};
use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use std::{net::SocketAddr, time::Duration};
use tokio::{
    net::{lookup_host, TcpListener, ToSocketAddrs},
    select,
    sync::{broadcast, mpsc},
    time::interval,
};
use tokio_tungstenite::{accept_async, MaybeTlsStream};
use tracing::*;
use vg_types::{PlayerEvent, PlayerId};

#[derive(Clone)]
struct Tick {
    state: Vec<u8>,
    events: Vec<PlayerEvent>,
    tickrate: Duration,
}

pub async fn run(addr: impl ToSocketAddrs, mut runtime: impl Runtime) -> Result<()> {
    let remote = lookup_host(addr).await?.next().unwrap();

    let listen = TcpListener::bind(remote).await?;
    debug!("Listening on {}", listen.local_addr()?);

    // let mut runtime = RT::load(&code)?;
    let mut id_source = IdSource::new();

    let mut handles = FuturesUnordered::new();
    let (tick_tx, _) = broadcast::channel::<Tick>(1024);
    let (event_tx, mut event_rx) = mpsc::channel::<PlayerEvent>(1024);

    let tickrate = Duration::from_millis(20);
    let mut ticker = interval(tickrate);

    let mut events = vec![];

    loop {
        select! {
            // Receive player events from active connections
            Some(ev) = event_rx.recv() => {
                events.push(ev);
            }
            // It is time for a TICK!
            _ = ticker.tick() => {
                let events = events.split_off(0);
                for ev in &events {
                    runtime.send(*ev);
                }

                runtime.run_tick(tickrate)?;
                if let Err(_err) = tick_tx.send(Tick { state: runtime.serialize()?, events, tickrate, }) {
                    //warn!("Failed to broadcast tick. Is server empty?: {}", err);
                }
            }
            // Accept new connections
            Ok((stream, addr)) = listen.accept() => {
                let mut socket = accept_async(MaybeTlsStream::Plain(stream)).await?;

                // Make sending events and ticks faster
                set_nodelay(&mut socket);

                let player = id_source.alloc();
                debug!("Accepted connection from {}, identity {:?}", addr, player);

                let tick_rx = tick_tx.subscribe();
                let event_tx = event_tx.clone();

                handles.push(tokio::spawn(async move {
                    if let Err(err) = handle(socket, addr, player, tick_rx, event_tx).await {
                        error!("Player {:?} disconnected with error: {}", player, err);
                    } else {
                        info!("Player {:?} disconnected", player);
                    }
                    player
                }));
            }
            // Accept leaves
            Some(id) = handles.next() => {
                match id {
                    Ok(id) => id_source.free(id),
                    Err(err) => warn!("Connection handler panicked! There is a zombie player now: {}", err),
                }
            }
        }
    }
}

async fn handle(
    mut socket: Socket,
    addr: SocketAddr,
    player: PlayerId,
    mut tick_rx: broadcast::Receiver<Tick>,
    event_tx: mpsc::Sender<PlayerEvent>,
) -> Result<()> {
    // let state = self.runtime.serialize()?;
    // Wait for next tick to send data
    let Tick { state, .. } = tick_rx.recv().await?;

    debug!("Sending sync to {} ({})", addr, Memory(state.len()));
    send(&mut socket, S2C::Sync { player, state }).await?;

    loop {
        select! {
            // Server tick occured, announce
            tick = tick_rx.recv() => {
                let Tick { events, tickrate, .. } = tick?;
                let tickrate_ns = tickrate.as_nanos() as u64;
                debug!("Sending tick to {:?}", player);
                send(&mut socket, S2C::Tick { events, tickrate_ns }).await?;
            }
            // Push client event into server event queue
            event = recv::<C2S>(&mut socket) => {
                let event = match event {
                    Ok(Some(e)) => e,
                    Ok(None) => break,
                    Err(err) => {
                        warn!("Error receiving C2S from {:?}: {}", player, err);
                        return Err(err)
                    }
                };
                debug!("Got event from {:?}: {:?}", player, event);
                match event {
                    C2S::Event(event) => {
                        event_tx.send(PlayerEvent {
                            player,
                            event
                        }).await?;
                    }
                }
            }
        }
    }

    // socket.flush().await?;

    Ok(())
}
