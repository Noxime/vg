use std::time::Duration;

use log::{debug, warn};

use crate::{util::TimedEvents, Game, PlayerEvent, PlayerId, Vg};

use super::{Conn, NetError, NetEvent, NetPlayer, C2S, S2C};

pub struct Client<G: Game> {
    state: Vg<G>,
    player: NetPlayer,
    pred_events: Vec<(Duration, PlayerEvent)>,
}

impl<G: Game> Client<G> {
    pub(crate) async fn new() -> Result<Self, NetError> {
        let addr = "127.0.0.1:5555";
        let conn = Conn::connect(addr.parse().unwrap()).await?;

        let mut player = NetPlayer {
            conn,
            id: PlayerId(0),
        };
        let state = Self::resync(&mut player).await?;

        Ok(Self {
            state,
            player,
            pred_events: vec![],
        })
    }

    // Request server for full state, buffer any in flight messages and then apply state
    async fn resync(player: &mut NetPlayer) -> Result<Vg<G>, NetError> {
        warn!("Desynced, requesting full state");
        player.conn.send_to_server(C2S::Desync).await?;

        loop {
            if let Some(S2C::Sync { state, id }) = player.conn.recv_from_server().await? {
                let state = bincode::deserialize(&state)?;
                player.id = id;
                return Ok(state);
            }
        }
    }

    pub(crate) async fn poll(&mut self) -> Result<Option<NetEvent<G>>, NetError> {
        match self.player.conn.recv_from_server().await? {
            Some(S2C::Sync {
                state: new_state,
                id,
            }) => {
                debug!("Unexpected resync, did it anyway");
                *self.state = bincode::deserialize(&new_state)?;
                self.player.id = id;
            }
            Some(S2C::Tick {
                mut events,
                delta,
                rollback,
                hash,
            }) => {
                debug!("Server tick received, {} events", events.len());

                self.pred_events.take_before(rollback);
                self.state.tick(events.take_before(rollback), delta);
                self.pred_events.tick(delta);

                // Because we receive ticks with rtt/2 latency, predict forwards n ticks to be closer in sync
                let mut prediction = self.state.duplicate();

                // events.extend(
                //     self.pred_events
                //         .drain(..)
                //         .map(|e| (Duration::ZERO, e)),
                // );

                events.extend_from_slice(&self.pred_events);

                for evs in events.rollback_events(rollback + self.latency(), delta) {
                    prediction.tick(evs, delta);
                }

                // check the resulting state for desyncs
                if self.state.hash() != hash {
                    self.state = Self::resync(&mut self.player).await?;
                }

                return Ok(Some(NetEvent::Tick(prediction, delta)));
            }
            None => (),
        }

        Ok(None)
    }

    pub async fn send(&mut self, event: PlayerEvent) {
        self.pred_events.push((Duration::ZERO, event.clone()));
        self.player
            .conn
            .send_to_server(C2S::Event { event: event.kind })
            .await
            .expect("Failed to send to server")
    }

    pub fn local_player(&self) -> PlayerId {
        self.player.id
    }

    pub fn ticks_behind(&self, delta: Duration) -> usize {
        self.latency().div_duration_f32(delta).floor() as usize
    }

    pub fn latency(&self) -> Duration {
        self.roundtrip() / 2
    }

    pub fn roundtrip(&self) -> Duration {
        self.player.conn.roundtrip()
    }

    pub fn traffic_tx(&self) -> usize {
        self.player.conn.traffic_tx()
    }

    pub fn traffic_rx(&self) -> usize {
        self.player.conn.traffic_rx()
    }

    pub fn ok(&self) -> bool {
        self.player.conn.ok()
    }
}
