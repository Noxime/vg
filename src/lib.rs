#![feature(
    arbitrary_self_types,
    duration_saturating_ops,
    async_closure,
    never_type,
    variant_count,
    associated_type_defaults,
    div_duration,
    duration_zero,
    drain_filter,
    specialization
)]
#![allow(incomplete_features)]

use assets::AssetLoader;
use event::{Digital, InputCache};
use log::*;
use network::{NetEvent, Network};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

use ultraviolet::{Mat4, Rotor3, Similarity3, Vec3};
use util::Lerp;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::{Duration, Instant};

mod event;
mod network;
mod util;
pub use event::{Event, Key, KeyEvent, PlayerEvent, PlayerId};
// mod serde_hash; TODO
pub mod assets;
pub mod gfx;
pub use gfx::{Model, Sprite};

// pub use ultraviolet::transform::Similarity3 as Transform;
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Rotor3,
    pub scale: f32,
}

impl Transform {
    pub fn to_mat(&self) -> Mat4 {
        Similarity3::new(self.position, self.rotation, self.scale).into_homogeneous_matrix()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: 1.0,
        }
    }
}

pub trait Game: Sized + Serialize + DeserializeOwned {
    type Player: Player<Self> = ();

    fn update(self: &mut Vg<Self>);

    fn hash(&self) -> u64 {
        fxhash::hash64(&bincode::serialize(self).unwrap())
    }
}

pub trait Player<G: Game>: Sized + Serialize + DeserializeOwned {
    fn connected(state: &mut Vg<G>, id: PlayerId) -> Self;
    fn disconnected(self, _state: &mut Vg<G>) {}
}

impl<G: Game> Player<G> for () {
    fn connected(_: &mut Vg<G>, _: PlayerId) -> Self {
        ()
    }
}

mod game_serde {
    use crate::Game;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, G: Game>(g: &G, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_bytes(&bincode::serialize(g).unwrap())
    }

    pub fn deserialize<'de, D, G: Game>(d: D) -> Result<G, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::deserialize(d)?;
        Ok(bincode::deserialize(&bytes).unwrap())
    }
}

mod player_serde {
    use crate::{event::InputCache, Game, PlayerId, VgPlayer};
    use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Alt {
        id: PlayerId,
        state: Vec<u8>,
        input: InputCache,
    }

    pub fn serialize<S, G: Game>(g: &Vec<VgPlayer<G>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut l = s.serialize_seq(Some(g.len()))?;

        for player in g {
            let bytes = bincode::serialize(&player.state).unwrap();
            let val = Alt {
                id: player.id,
                state: bytes,
                input: player.input.clone(),
            };
            l.serialize_element(&val)?;
        }

        l.end()
    }

    pub fn deserialize<'de, D, G: Game>(d: D) -> Result<Vec<VgPlayer<G>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: Vec<Alt> = Vec::deserialize(d)?;
        let mut list = vec![];

        for alt in map {
            let state = bincode::deserialize(&alt.state).unwrap();
            list.push(VgPlayer {
                id: alt.id,
                input: alt.input,
                state,
            })
        }

        Ok(list)
    }
}

// #[derive(Serialize, Deserialize)]
pub struct VgPlayer<G: Game> {
    id: PlayerId,
    // #[serde(with = "player_serde")]
    state: G::Player,
    input: InputCache,
}

impl<G: Game> Clone for VgPlayer<G> {
    fn clone(&self) -> Self {
        VgPlayer {
            id: self.id,
            input: self.input.clone(),
            state: bincode::deserialize(&bincode::serialize(&self.state).unwrap()).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Vg<G: Game> {
    run_time: Duration,
    delta_time: Duration,
    #[serde(with = "game_serde")]
    state: G,
    #[serde(with = "player_serde")]
    players: Vec<VgPlayer<G>>,
}

pub fn run(state: impl Game + 'static) {
    let mut vg = Vg::new(state);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("vg-")
        .build()
        .unwrap();

    let mut network = rt.block_on(Network::new(vg.duplicate())).unwrap();

    let ev = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("VG")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .build(&ev)
        .unwrap();

    let mut gfx = rt.block_on(gfx::Gfx::new(&window));
    let mut assets = AssetLoader::new();

    let mut last_frame = Instant::now();
    let mut fps = 0.0f32;
    let mut last_tick = Instant::now();
    let mut tps = 0.0f32;

    // queue for our prediction events
    // let mut events = vec![];

    ev.run(move |event, _, flow| {
        use winit::event::{Event as WEvent, WindowEvent};
        *flow = ControlFlow::Poll;

        if !network.client.ok() {
            info!("Exiting (connection closed)");
            *flow = ControlFlow::Exit;
            return;
        }

        // Convert winit event into a VG event
        if let Some(kind) = Event::new(&event) {
            let event = PlayerEvent {
                player: network.client.local_player(),
                kind: kind.clone(),
            };

            // Transmit our event to the host
            rt.block_on(network.client.send(event.clone()));
            // Apply the event to our local prediction state, but n ticks in the future as thats
            // when our event will arrive at the host
            // events.push((Instant::now() + network.client.latency(), event));
        }

        match event {
            // All events are cleared, now we can run all the book keeping
            WEvent::MainEventsCleared => match rt.block_on(network.poll()) {
                Ok(Some(NetEvent::Tick(ns, delta))) => {
                    let tick_time = last_tick.elapsed();
                    last_tick = Instant::now();
                    tps.lerp(tick_time.as_secs_f32(), 0.05);
                    vg = ns;

                    // This code attempts to synchonize the client and host, as client is normally rtt/2 behind the
                    // host. It works, but also causes noticeable popping

                    // for tick in 0..network.client.ticks_behind(delta) {
                    //     let pred_index = events
                    //         .iter()
                    //         .position(|(i, _)| (*i - delta * tick as u32).is_future())
                    //         .unwrap_or(events.len());
                    //     let mut pred_evs = events.split_off(pred_index);
                    //     std::mem::swap(&mut pred_evs, &mut events);

                    //     let pred_evs = pred_evs.into_iter().map(|(_, e)| e).collect();
                    //     vg.tick(pred_evs, delta);
                    // }
                }
                // Everything done for now, run aux tasks
                Ok(None) => {
                    let frame_time = last_frame.elapsed();
                    last_frame = Instant::now();
                    // Smooth the FPS a little
                    fps.lerp(frame_time.as_secs_f32(), 0.01);
                    window.set_title(&format!(
                        "VG | {:.1} fps | {:.1} tps | {} ms | rx {} kb / tx {} kb",
                        1.0 / fps,
                        1.0 / tps,
                        network.client.roundtrip().as_millis(),
                        network.client.traffic_rx() / 1024,
                        network.client.traffic_tx() / 1024,
                    ));

                    // Run local prediction
                    // This grabs all the events from the prediction events that occurred in the past (+ latency) and runs
                    // game tick with them
                    // let pred_index = events
                    //     .iter()
                    //     .position(|(i, _)| i.is_future())
                    //     .unwrap_or(events.len());
                    // let mut pred_evs = events.split_off(pred_index);
                    // std::mem::swap(&mut pred_evs, &mut events);

                    // let pred_evs = pred_evs.into_iter().map(|(_, e)| e).collect();
                    // vg.tick(pred_evs, frame_time);

                    // vg.tick(vec![], frame_time);

                    rt.block_on(gfx.draw(&mut vg, &mut assets))
                        .expect("Render error");
                }
                Err(err) => {
                    error!("Networking error occurred: {:?}", err);
                }
            },
            // Respond to framebuffer resize
            WEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                gfx.resize(size.width, size.height);
            }
            // Clean exit
            WEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Exiting");
                *flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

impl<G: Game> Vg<G> {
    pub fn new(state: G) -> Vg<G> {
        Vg {
            run_time: Duration::default(),
            delta_time: Duration::default(),
            state,
            players: vec![],
            // events: vec![],
        }
    }

    pub fn duplicate(&self) -> Vg<G> {
        Vg {
            run_time: self.run_time,
            delta_time: self.delta_time,
            players: self.players.clone(),
            // events: self.events.clone(),
            state: bincode::deserialize(&bincode::serialize(&self.state).unwrap()).unwrap(),
        }
    }

    // fn process_netev(&mut self, event: NetEvent<G>) {
    //     match event {
    //         NetEvent::Tick(new) => *self = new,
    //     }
    // }

    fn process_event(&mut self, event: PlayerEvent) {
        if let Some(i) = self.players().iter().position(|p| p.id == event.player) {
            self.players_mut()[i].input.event(&event.kind);
        }

        match event.kind {
            Event::Connected => {
                let state = G::Player::connected(self, event.player);
                self.players.push(VgPlayer {
                    id: event.player,
                    state,
                    input: InputCache::new(),
                });
            }
            Event::Disconnected => {
                if let Some(i) = self.players.iter().position(|p| p.id == event.player) {
                    let player = self.players.swap_remove(i);
                    G::Player::disconnected(player.state, self);
                }
            }
            _ => (),
        }
    }

    fn advance_time(&mut self, delta: Duration) {
        self.run_time += delta;
        self.delta_time = delta;
    }

    fn tick(&mut self, events: Vec<PlayerEvent>, delta: Duration) {
        for e in events {
            self.process_event(e);
        }

        for p in self.players_mut() {
            p.input.tick();
        }

        self.advance_time(delta);
        Game::update(self);
    }

    /* public API */

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn run_time(&self) -> Duration {
        self.run_time
    }

    pub fn players(&self) -> &[VgPlayer<G>] {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut [VgPlayer<G>] {
        &mut self.players
    }
}

impl<G: Game> VgPlayer<G> {
    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn key(&self, key: Key) -> Digital {
        self.input.key(key)
    }

    pub fn wasd_arrows(&self) -> Vec3 {
        self.input.wasd_arrows()
    }
}

impl<G: Game> std::ops::Deref for VgPlayer<G> {
    type Target = G::Player;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<G: Game> std::ops::DerefMut for VgPlayer<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<G: Game> std::ops::Deref for Vg<G> {
    type Target = G;
    fn deref(&self) -> &Self::Target {
        &self.state
        // &self.universe().state
    }
}

impl<G: Game> std::ops::DerefMut for Vg<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
