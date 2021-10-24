use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    debug::Memory,
    net::{
        event_queue::EventQueue, http, Channel, Nothing, Ping, Pong, Tick, MAX_MESSAGE_SIZE,
        MAX_RECEIVE_BUFFER_SIZE,
    },
    runtime::Runtime,
};
use anyhow::Result;
use futures::{
    future::{join_all, ready},
    stream::FuturesUnordered,
};
use std::sync::mpsc::Sender as SyncSender;
use tokio::{select, sync::mpsc::Receiver};
use tracing::*;
use vg_types::{Event, PlayerEvent, PlayerId};
use webrtc_data::data_channel::{self, DataChannel};
use webrtc_sctp::{
    association::{self, Association},
    chunk::chunk_payload_data::PayloadProtocolIdentifier,
};

pub async fn run<RT: Runtime>(
    addr: IpAddr,
    mut rx: Receiver<Event>,
    tx: SyncSender<RT>,
) -> Result<()> {
    let (socket, state) = http::req_socket(addr).await?;
    debug!("Client socket established");

    let mut runtime = RT::deserialize(&state)?;
    let mut event_queue = EventQueue::new();

    let association = Arc::new(
        Association::client(association::Config {
            net_conn: Arc::new(socket),
            max_receive_buffer_size: MAX_RECEIVE_BUFFER_SIZE,
            max_message_size: MAX_MESSAGE_SIZE,
            name: "ClientAssociation".into(),
        })
        .await?,
    );

    let ping_channel: Channel<Ping, Pong, false> =
        Channel::accept(DataChannel::accept(&association, Default::default()).await?).unwrap();

    let event_channel: Channel<u64, EventQueue, false> =
        Channel::accept(DataChannel::accept(&association, Default::default()).await?).unwrap();

    let tick_channel: Channel<Tick, Nothing, true> =
        Channel::accept(DataChannel::accept(&association, Default::default()).await?).unwrap();

    loop {
        select! {
            // Reply to ping requests
            Ok(Ping(challenge)) = ping_channel.recv() => {
                debug!("Replying to ping {}", challenge);
                ping_channel.send(Pong(challenge)).await?;
            },
            // Broadcast our event
            Some(ev) = rx.recv() => {
                event_queue.push(ev);
            }
            // Clear events that the server has already acked
            Ok(i) = event_channel.recv() => {
                debug!("Server acknowledged up to {}", i);
                event_queue.ack(i);
            }
            Ok(Tick { events, tickrate_ns, }) = tick_channel.recv() => {
                let tickrate = Duration::from_nanos(tickrate_ns);

                for ev in events {
                    runtime.send(ev);
                }
                let _calls = runtime.run_tick(tickrate)?;
                
                tx.send(runtime.duplicate()?).unwrap();
            }
        }

        if !event_queue.empty() {
            debug!("Sending events {:?}", event_queue.range());
            event_channel.send(event_queue.clone()).await?;
        }
    }

    // let dc = DataChannel::dial(&association, 1337, config()).await?;

    info!("Disconnected from server");

    Ok(())
}
