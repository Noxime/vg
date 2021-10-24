use std::{iter::FromIterator, net::Ipv4Addr, sync::Arc, time::Duration};

use crate::{
    debug::{Memory, KB},
    ids::IdSource,
    net::{
        event_queue::EventQueue, http, Channel, Nothing, Ping, Pong, Tick, MAX_MESSAGE_SIZE,
        MAX_RECEIVE_BUFFER_SIZE,
    },
    runtime::Runtime,
};
use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    net::UdpSocket,
    select,
    sync::{broadcast, mpsc},
    time::{interval, Instant},
};
use tracing::*;
use vg_types::{PlayerEvent, PlayerId};
use webrtc_data::data_channel::DataChannel;
use webrtc_sctp::association::{self, Association};

#[derive(Clone)]
pub struct FullTick {
    pub state: Vec<u8>,
    pub events: Vec<PlayerEvent>,
    pub tickrate: Duration,
}

pub async fn run(mut runtime: impl Runtime) -> Result<()> {
    let (addr_tx, mut addr_rx) = mpsc::channel(16);
    let (tick_tx, tick_rx) = broadcast::channel(16);
    let (event_tx, mut event_rx) = mpsc::channel(16);
    tokio::spawn(http::run(
        Ipv4Addr::new(0, 0, 0, 0).into(),
        addr_tx,
        tick_tx.clone(),
    ));

    let mut tasks = FuturesUnordered::new();
    let mut player_alloc = IdSource::new();

    let tickrate = Duration::from_millis(1000);
    let mut ticker = interval(tickrate);

    let mut events = vec![];

    // Run accept and cleanup loop
    loop {
        select! {
            // It is time for a server tick
            _ = ticker.tick() => {
                let events = events.split_off(0);

                for ev in &events {
                    runtime.send(*ev);
                }
                runtime.run_tick(tickrate)?;
                let state = runtime.serialize()?;


                // Send will fail when there are 0 players connected, but that is fine
                let _ = tick_tx.send(FullTick {
                    state,
                    events,
                    tickrate,
                });
            }
            // New connection
            Some(socket) = addr_rx.recv() => {
                let player = player_alloc.alloc();
                let tick_rx = tick_tx.subscribe();
                let event_tx = event_tx.clone();
                tasks.push(tokio::spawn(async move {
                    match handle(player, socket, tick_rx, event_tx).await {
                        Ok(()) => info!("Player {:?} disconnected", player),
                        Err(err) => warn!("Player {:?} disconnected with error: {}", player, err)
                    }
                    player
                }));
            },
            // Connection closed
            Some(task) = tasks.next() => {
                match task {
                    Ok(player) => player_alloc.free(player),
                    Err(err) => error!("Connection handler paniced (Zombie player created): {}", err),
                }
            }
            // Player event
            Some(event) = event_rx.recv() => {
                events.push(event);
            }
        }
    }
}

async fn handle(
    player: PlayerId,
    socket: UdpSocket,
    mut tick_rx: broadcast::Receiver<FullTick>,
    mut event_tx: mpsc::Sender<PlayerEvent>,
) -> Result<()> {
    let association = Arc::new(
        Association::client(association::Config {
            net_conn: Arc::new(socket),
            max_receive_buffer_size: MAX_RECEIVE_BUFFER_SIZE,
            max_message_size: MAX_MESSAGE_SIZE,
            name: "ClientAssociation".into(),
        })
        .await?,
    );

    let ping_channel: Channel<Pong, Ping, false> = Channel::open(&association).await?;
    let event_channel: Channel<EventQueue, u64, false> = Channel::open(&association).await?;
    let tick_channel: Channel<Nothing, Tick, true> = Channel::open(&association).await?;

    let FullTick { state, .. } = tick_rx.recv().await?;

    info!("Channels connected to player {:?}", player);

    let mut pinger = interval(Duration::from_secs(1));
    let mut ping_start = Instant::now();

    let mut seq = 0;

    loop {
        select! {
            // Dispatch a ping every second
            instant = pinger.tick() => {
                ping_start = instant;
                ping_channel.send(Ping(0)).await?;
            },
            // Calculate round trip
            Ok(Pong(_)) = ping_channel.recv() => {
                debug!("RTT: {:?}", ping_start.elapsed());
            },
            Ok(FullTick { events, tickrate, .. }) = tick_rx.recv() => {
                debug!("Sending events to {:?}", player);
                let tickrate_ns = tickrate.as_nanos() as u64;
                tick_channel.send(Tick { events, tickrate_ns }).await?;
            },
            // Receive events from client
            Ok(events) = event_channel.recv() => {
                debug!("Got events {:?} for {:?}", events.range(), player);

                for &event in events.after(seq) {
                    event_tx.send(PlayerEvent {
                        player,
                        event,
                    }).await?;
                }

                seq = events.up_to();
                event_channel.send(seq).await?;

            },
        }
    }
}
