use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    time::{Duration, Instant},
};

use futures::executor::block_on;
use laminar::{Packet, Socket, SocketEvent};
use lazy_static::lazy_static;
use log::{debug, info, trace};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tungstenite::{
    client::AutoStream,
    handshake::{
        client::Request as ClientRequest,
        server::{Request as ServerRequest, Response},
    },
    stream::NoDelay,
    util::NonBlockingResult,
    Message, WebSocket,
};

use tokio::sync::{mpsc, oneshot, Mutex};

use crate::Event;

use super::{NetError, C2S, S2C};

type LocalChannel = (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>);

lazy_static! {
    static ref SERVER_TX: Mutex<Option<mpsc::Sender<oneshot::Sender<LocalChannel>>>> =
        Mutex::new(None);
}

// Try awaiting the future, return None if did not ready
async fn nonb<T>(f: impl std::future::Future<Output = T>) -> Option<T> {
    tokio::time::timeout(Duration::ZERO, f).await.ok()
}

pub(crate) struct Listener {
    websocket: TcpListener,
    rx: mpsc::Receiver<oneshot::Sender<LocalChannel>>,
}

impl Listener {
    pub async fn bind(addr: &str) -> Result<Self, NetError> {
        // TODO: Figure out if we should actually just bind port and port+1
        let websocket = TcpListener::bind(addr)?;
        info!("Listening on TCP/WS: {}", websocket.local_addr()?,);

        // We don't want to wait for network
        websocket.set_nonblocking(true)?;

        let (tx, rx) = mpsc::channel(10);

        let mut lock = SERVER_TX.lock().await;
        *lock = Some(tx);
        drop(lock);

        Ok(Self { websocket, rx })
    }

    pub async fn accept(&mut self) -> Result<Option<Conn>, NetError> {
        if let Ok(msg) = self.rx.try_recv() {
            debug!("Accepted local connection");

            let (atx, arx) = mpsc::channel(100);
            let (btx, brx) = mpsc::channel(100);

            msg.send((atx, brx)).unwrap();

            return Ok(Some(Conn::Local { rx: arx, tx: btx }));
        }

        if let Some((stream, mut addr)) = self.websocket.accept().no_block()? {
            stream.set_nodelay(true)?;
            stream.set_nonblocking(true)?;

            let stream = tungstenite::stream::Stream::Plain(stream);

            let (tx, rx) = oneshot::channel();

            let cb = |req: &ServerRequest, mut res: Response| {
                info!("Accepted WS: {}", addr);

                if let Some(Ok(Ok(client_port))) = req
                    .headers()
                    .get("Vg-Client-Laminar-Port")
                    .map(|h| h.to_str().map(|s| s.parse()))
                {
                    addr.set_port(client_port);
                    let laminar = Socket::bind_any().unwrap();
                    let server_port = laminar.local_addr().unwrap().port();

                    let headers = res.headers_mut();
                    headers.append("Vg-Server-Laminar-Port", server_port.into());

                    debug!("Laminar server connected to: {}", addr);

                    tx.send(Some((addr, laminar))).unwrap()
                } else {
                    tx.send(None).unwrap()
                }

                Ok(res)
            };

            if let Ok(ws) = tungstenite::accept_hdr(stream, cb) {
                return Ok(Some(Conn::from_ws(ws, rx.await?)));
            }
        }

        Ok(None)
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        block_on(async {
            SERVER_TX.lock().await.take(); // remove the receiver
        });
    }
}

pub(crate) enum Conn {
    Ws {
        ws: WebSocket<AutoStream>,
        laminar: Option<Laminar>,
        rtt: Duration,
        ping: Instant,
        bytes_send: usize,
        bytes_recv: usize,
    },
    Local {
        rx: mpsc::Receiver<Vec<u8>>,
        tx: mpsc::Sender<Vec<u8>>,
    },
}

pub(crate) struct Laminar {
    addr: SocketAddr,
    socket: Socket,
    send_events: VecDeque<Event>,
    recv_events: VecDeque<Event>,
    send_seq: u64,
    recv_seq: u64,
}

impl Laminar {
    fn from_opt(opt: Option<(SocketAddr, Socket)>) -> Option<Self> {
        if let Some((addr, socket)) = opt {
            Some(Laminar {
                addr,
                socket,
                send_events: VecDeque::new(),
                recv_events: VecDeque::new(),
                send_seq: 0,
                recv_seq: 0,
            })
        } else {
            None
        }
    }

    fn poll(&mut self) {
        self.socket.manual_poll(Instant::now())
    }

    fn recv_from_client(&mut self) -> Result<Option<C2S>, NetError> {
        if let Some(SocketEvent::Packet(packet)) = self.socket.recv() {
            let payload = bincode::deserialize(packet.payload())?;

            match payload {
                C2SLaminar::Event { seq, mut events } if seq > self.recv_seq => {

                    let discard = events.len() as u64 - (seq - self.recv_seq);

                    for _ in 0..discard {
                        events.pop_front();
                    }

                    self.recv_events.append(&mut events);

                    trace!("Receive event queue: {}", self.recv_events.len());

                    let payload = bincode::serialize(&S2CLaminar::EventAck { seq })?;
                    self.socket.send(Packet::unreliable(self.addr, payload))?;

                    self.recv_seq = seq;
                },
                C2SLaminar::Event { .. } => () // Out of order, late packet
            }
        }

        Ok(self
            .recv_events
            .pop_front()
            .map(|event| C2S::Event { event }))
    }

    fn recv_from_server(&mut self) -> Result<Option<S2C>, NetError> {
        if let Some(SocketEvent::Packet(packet)) = self.socket.recv() {
            let payload = bincode::deserialize(packet.payload())?;

            match payload {
                S2CLaminar::EventAck { seq } => {
                    let num_not_acked = self.send_seq - seq;

                    // Disregard the events that are delivered properly
                    while self.send_events.len() as u64 > num_not_acked {
                        self.send_events.pop_front();
                    }
                }
            }
        }

        Ok(None)
    }

    fn send_to_client(&mut self, _: &S2C) -> Result<bool, NetError> {
        Ok(false)
    }

    fn send_to_server(&mut self, msg: &C2S) -> Result<bool, NetError> {
        match msg {
            C2S::Event { event } => {
                // This is a lower latency, higher bandwidth version of regular event sending
                // Essentially, we send a buffer of old events, which gets shortened every time
                // we receive an Ack.
                self.send_events.push_back(event.clone());
                self.send_seq += 1;

                trace!("Send event queue: {}", self.send_events.len());

                let payload = bincode::serialize(&C2SLaminar::Event {
                    seq: self.send_seq,
                    events: self.send_events.clone(),
                })?;

                self.socket.send(Packet::unreliable(self.addr, payload))?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) enum S2CLaminar {
    EventAck { seq: u64 },
}

#[derive(Serialize, Deserialize)]
pub(crate) enum C2SLaminar {
    Event { seq: u64, events: VecDeque<Event> },
}

impl Conn {
    fn from_ws(ws: WebSocket<AutoStream>, laminar: Option<(SocketAddr, Socket)>) -> Conn {
        Conn::Ws {
            ws,
            laminar: Laminar::from_opt(laminar),
            rtt: Duration::default(),
            ping: Instant::now(),
            bytes_send: 0,
            bytes_recv: 0,
        }
    }

    pub async fn connect(mut addr: SocketAddr) -> Result<Self, NetError> {
        if addr.ip().is_loopback() {
            debug!("Loopback connection found, trying channel connection");
            if let Some(tx) = SERVER_TX.lock().await.clone() {
                let (otx, orx) = oneshot::channel();
                tx.send(otx).await.unwrap();

                let (tx, rx) = orx.await?;

                return Ok(Self::Local { rx, tx });
            }
        }

        let laminar = Socket::bind("0.0.0.0:0")?;

        let mut request = ClientRequest::new(());
        *request.uri_mut() = format!("ws://{}", addr).parse().unwrap();

        let headers = request.headers_mut();
        headers.append(
            "Vg-Client-Laminar-Port",
            laminar.local_addr()?.port().into(),
        );

        let (mut ws, res) = tungstenite::connect(request)?;
        info!(
            "WS connected to {} ({:?} {})",
            addr,
            res.version(),
            res.status()
        );

        let laminar = if let Some(Ok(Ok(server_port))) = res
            .headers()
            .get("Vg-Server-Laminar-Port")
            .map(|h| h.to_str().map(|s| s.parse()))
        {
            addr.set_port(server_port);
            debug!("Laminar client connected to {}", addr);
            Some((addr, laminar))
        } else {
            None
        };

        ws.get_mut().set_nodelay(true)?;
        match ws.get_mut() {
            tungstenite::stream::Stream::Plain(s) => s.set_nonblocking(true)?,
            tungstenite::stream::Stream::Tls(s) => s.get_ref().set_nonblocking(true)?,
        }

        ws.write_message(Message::Ping(vec![]))?;

        Ok(Self::from_ws(ws, laminar))
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, NetError> {
        match self {
            Self::Ws { ws, .. } => match ws.get_ref() {
                tungstenite::stream::Stream::Plain(s) => Ok(s.peer_addr()?),
                tungstenite::stream::Stream::Tls(s) => Ok(s.get_ref().peer_addr()?),
            },
            Self::Local { .. } => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)),
        }
    }

    pub fn latency(&self) -> Duration {
        self.roundtrip() / 2
    }

    pub fn roundtrip(&self) -> Duration {
        match self {
            Self::Ws { rtt, .. } => *rtt,
            Self::Local { .. } => Duration::ZERO, // Local connection, no latency possible
        }
    }

    pub fn traffic_tx(&self) -> usize {
        match self {
            Self::Ws { bytes_send, .. } => *bytes_send,
            Self::Local { .. } => 0,
        }
    }

    pub fn traffic_rx(&self) -> usize {
        match self {
            Self::Ws { bytes_recv, .. } => *bytes_recv,
            Self::Local { .. } => 0,
        }
    }

    pub fn ok(&self) -> bool {
        match self {
            Self::Ws { ws, .. } => ws.can_read() && ws.can_write(),
            Self::Local { .. } => true,
        }
    }

    async fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>, NetError> {
        match self {
            Self::Ws {
                ws,
                rtt,
                ping,
                bytes_recv,
                ..
            } => match ws.read_message().no_block()? {
                Some(Message::Binary(bytes)) => {
                    *bytes_recv += bytes.len();
                    // let bytes = lz4_flex::decompress_size_prepended(&bytes)?;
                    Ok(Some(bincode::deserialize(&bytes)?))
                }
                Some(Message::Pong(_)) => {
                    *rtt = ping.elapsed();
                    *ping = Instant::now();
                    // After pong, send ping immediately
                    ws.write_message(Message::Ping(vec![]))?;
                    Ok(None)
                }
                _ => Ok(None),
            },
            Self::Local { rx, .. } => {
                if let Some(bytes) = nonb(rx.recv()).await.flatten() {
                    Ok(Some(bincode::deserialize(&bytes)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn poll(&mut self) {
        if let Conn::Ws {
            laminar: Some(s), ..
        } = self
        {
            s.poll();
        }
    }

    pub async fn recv_from_client(&mut self) -> Result<Option<C2S>, NetError> {
        self.poll();
        if let Self::Ws {
            laminar: Some(laminar),
            ..
        } = self
        {
            if let Some(msg) = laminar.recv_from_client()? {
                return Ok(Some(msg));
            }
        }
        self.recv().await
    }

    pub async fn recv_from_server(&mut self) -> Result<Option<S2C>, NetError> {
        self.poll();
        if let Self::Ws {
            laminar: Some(laminar),
            ..
        } = self
        {
            if let Some(msg) = laminar.recv_from_server()? {
                return Ok(Some(msg));
            }
        }
        self.recv().await
    }

    async fn send<T: Serialize>(&mut self, msg: T) -> Result<(), NetError> {
        match self {
            Self::Ws { ws, bytes_send, .. } => {
                let bytes = bincode::serialize(&msg)?;
                let size = bytes.len();
                // let bytes = lz4_flex::compress_prepend_size(&bytes);
                trace!(
                    "Sending {} bytes (before compression {})",
                    bytes.len(),
                    size
                );
                *bytes_send += bytes.len();
                ws.write_message(Message::Binary(bytes)).no_block()?;
                Ok(())
            }
            Self::Local { tx, .. } => {
                let bytes = bincode::serialize(&msg)?;
                tx.send(bytes).await?;
                Ok(())
            }
        }
    }

    pub async fn send_to_client(&mut self, msg: S2C) -> Result<(), NetError> {
        self.poll();

        if let Self::Ws {
            laminar: Some(laminar),
            ..
        } = self
        {
            if !laminar.send_to_client(&msg)? {
                self.send(msg).await?
            }
        } else {
            self.send(msg).await?
        }

        Ok(())
    }

    pub async fn send_to_server(&mut self, msg: C2S) -> Result<(), NetError> {
        self.poll();

        if let Self::Ws {
            laminar: Some(laminar),
            ..
        } = self
        {
            if !laminar.send_to_server(&msg)? {
                self.send(msg).await?
            }
        } else {
            self.send(msg).await?
        }

        Ok(())
    }
}
