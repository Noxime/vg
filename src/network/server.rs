use std::time::{Duration, Instant};

use log::{debug, info, trace, warn};

use crate::{
    util::{IsFuture, TimedEvents},
    Event, Game, PlayerEvent, PlayerId, Vg,
};

use super::{Listener, NetError, NetPlayer, C2S, S2C};

pub(crate) struct Server<G: Game> {
    listener: Listener,
    state: Vg<G>,
    rollback: Duration,
    tick_rate: Duration,
    next_tick: Instant,
    players: Vec<NetPlayer>,
    events: Vec<(Duration, PlayerEvent)>, // Events and how long ago did they occur
}

impl<G: Game> Server<G> {
    pub async fn new(state: Vg<G>) -> Result<Self, NetError> {
        let listener = Listener::bind("0.0.0.0:5555").await?;

        let mut s = Self {
            listener,
            state,
            rollback: Duration::from_millis(500),
            tick_rate: Duration::from_secs(1) / 10,
            next_tick: Instant::now(),
            players: vec![],
            events: vec![],
        };

        // We need rollback's amount of history before we can start potentially rolling back
        for _ in 0..s.rollback_ticks() {
            s.state.tick(vec![], s.tick_rate);
        }

        Ok(s)
    }

    pub async fn poll(&mut self) -> Result<(), NetError> {
        // Go through every client and keep the connected ones
        let players = self.players.split_off(0);
        for client in players {
            // Is the connection alive
            if client.conn.ok() && client.conn.latency() < self.rollback {
                self.players.push(client);
            } else {
                self.events.push((
                    client.conn.latency(),
                    PlayerEvent {
                        player: client.id,
                        kind: Event::Disconnected,
                    },
                ));
            }
        }

        if !self.next_tick.is_future() {
            self.next_tick += self.tick_rate;
            trace!("Server tick!");
            if !self.next_tick.is_future() {
                warn!("Can't keep up! Did the system time change, or is the server overloaded?");
            }

            let all_events = self.events.clone();

            self.state
                .tick(self.events.take_before(self.rollback), self.tick_rate);
            let hash = self.state.hash();

            for client in &mut self.players {
                client
                    .conn
                    .send_to_client(S2C::Tick {
                        events: all_events.clone(),
                        delta: self.tick_rate,
                        rollback: self.rollback,
                        hash,
                    })
                    .await?;
            }

            self.events.tick(self.tick_rate);
            // self.events.clear();
        }

        // Accept new connections
        if let Some(mut conn) = self.listener.accept().await? {
            let bytes = bincode::serialize(&self.state)?;
            let id = PlayerId::from_hash(conn.peer_addr()?);
            debug!("Sending initial sync ({}kb)", bytes.len() / 1024);
            conn.send_to_client(S2C::Sync { state: bytes, id }).await?;

            self.events.push((
                conn.latency(),
                PlayerEvent {
                    player: id,
                    kind: Event::Connected,
                },
            ));

            self.players.push(NetPlayer { conn, id });
        }

        for client in &mut self.players {
            match client.conn.recv_from_client().await? {
                Some(C2S::Event { event }) => {
                    self.events.push((
                        client.conn.latency(),
                        PlayerEvent {
                            player: client.id,
                            kind: event,
                        },
                    ));
                }
                Some(C2S::Desync) => {
                    warn!(
                        "Client {} announced it was desynced, send full state",
                        client.id
                    );
                    let bytes = bincode::serialize(&self.state)?;
                    client
                        .conn
                        .send_to_client(S2C::Sync {
                            state: bytes,
                            id: client.id,
                        })
                        .await?;
                }
                None => (),
            }
        }

        Ok(())
    }

    fn rollback_ticks(&self) -> usize {
        self.rollback.div_duration_f32(self.tick_rate).ceil() as usize
    }
}
