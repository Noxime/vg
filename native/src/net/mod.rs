mod client;
mod event_queue;
mod http;
mod server;

use crate::debug::{KB, MB};
use anyhow::Result;
use bytes::Bytes;
pub use client::run as client;
use derivative::Derivative;
use event_queue::EventQueue;
use futures::{SinkExt, StreamExt};
pub use server::run as server;
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tracing::{debug, trace, warn};
use vg_types::{DeBin, SerBin};
use vg_types::{Event, PlayerEvent, PlayerId};
use webrtc_data::data_channel::{self, DataChannel};
use webrtc_data::message::message_channel_open::ChannelType;
use webrtc_sctp::association::{self, Association};

const MAX_RECEIVE_BUFFER_SIZE: u32 = 64 * KB as u32;
const MAX_MESSAGE_SIZE: u32 = 64 * KB as u32;

const HTTP_PORT: u16 = 6502;

#[derive(SerBin, DeBin, Debug)]
struct Ping(u64);

#[derive(SerBin, DeBin, Debug)]
struct Pong(u64);

#[derive(SerBin, DeBin, Debug)]
struct Nothing;

#[derive(SerBin, DeBin, Debug)]
struct Tick {
    events: Vec<PlayerEvent>,
    tickrate_ns: u64,
}

pub struct Channel<I, O, const RELIABLE: bool> {
    dc: DataChannel,
    __phantom: PhantomData<(I, O)>,
}

impl<I: SerBin + DeBin, O: SerBin + DeBin, const RELIABLE: bool> Channel<I, O, RELIABLE>
where
    Self: 'static,
{
    // Accept a data channel that matches these settings
    pub fn accept(dc: DataChannel) -> Result<Self, DataChannel> {
        // Ensure the channel is for this message type
        if Self::id_opposite() != dc.stream_identifier() {
            debug!(
                "Channel ID does not match target, {} != {}",
                dc.stream_identifier(),
                Self::id()
            );
            return Err(dc);
        }

        // Ensure the channel has same reliability requirement
        if dc.config.channel_type != Self::channel_type() {
            return Err(dc);
        }

        Ok(Channel {
            dc,
            __phantom: PhantomData,
        })
    }

    pub async fn open(association: &Arc<Association>) -> Result<Self> {
        let dc = DataChannel::dial(
            association,
            Self::id(),
            data_channel::Config {
                channel_type: Self::channel_type(),
                reliability_parameter: 0,
                ..Default::default()
            },
        )
        .await?;

        Ok(Channel {
            dc,
            __phantom: PhantomData,
        })
    }

    pub async fn send(&self, msg: O) -> Result<()> {
        let bytes = msg.serialize_bin();
        trace!("Sending {} bytes", bytes.len());
        assert_eq!(
            bytes.len(),
            self.dc.write(&Bytes::from(bytes)).await?,
            "Tried sending message which was too big"
        );
        Ok(())
    }

    pub async fn recv(&self) -> Result<I> {
        let mut buf = vec![0; MAX_MESSAGE_SIZE as usize];
        let len = self.dc.read(&mut buf).await?;
        trace!("Received {} bytes", len);
        let bytes = &buf[..len];
        Ok(I::deserialize_bin(bytes)?)
    }

    fn id() -> u16 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish() as u16
    }

    fn id_opposite() -> u16 {
        Channel::<O, I, RELIABLE>::id()
    }

    fn channel_type() -> ChannelType {
        if RELIABLE {
            ChannelType::Reliable
        } else {
            ChannelType::PartialReliableRexmitUnordered
        }
    }
}
