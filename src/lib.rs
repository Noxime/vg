#![feature(
    arbitrary_self_types,
    duration_saturating_ops,
    async_closure,
    never_type,
    variant_count,
    // associated_type_defaults,
)]

use assets::AssetLoader;
use event::InputCache;
use log::*;
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ultraviolet::{Mat4, Rotor3, Similarity3, Vec3};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::{Duration, Instant};

mod network;
use network::Network;
mod event;
pub use event::{Event, EventKind, Key, KeyEvent, PlayerId};
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
    fn update(self: &mut Vg<Self>);
}

#[derive(Debug)]
struct Options {
    tick_rate: usize,
    predict_rate: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tick_rate: 30,
            predict_rate: 1,
        }
    }
}

#[derive(Eq, PartialEq)]
enum Tick {
    Real,
    Prediction,
}


pub struct Vg<G> {
    real: Universe<G>,
    prediction: Universe<G>,
    // are we real or prediction tick
    tick: Tick,
    options: Options,
    network: Network,
    // events queued up for next tick
    event_queue: Vec<EventKind>,

    local_player: PlayerId,
    players: Vec<Option<PlayerId>>,
    assets: AssetLoader,
    // debug_menu: gfx::ui::Ui,
}

struct Universe<G> {
    // game state
    state: G,
    // events for this tick
    events: Vec<Event>,
    // the current state of game inputs
    input_cache: InputCache,
}

impl<G: Game> Clone for Universe<G> {
    fn clone(&self) -> Self {
        let bin = bincode::serialize(&self.state).unwrap();
        let state = bincode::deserialize(&bin).unwrap();

        Self {
            state,
            events: self.events.clone(),
            input_cache: self.input_cache.clone(),
        }
    }
}

impl<G: Game> Vg<G> {
    pub fn run(state: G)
    where
        G: 'static,
    {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("vg-")
            .build()
            .unwrap();

        let vg = rt.block_on(async {
            let state = Universe {
                state,
                events: vec![],
                input_cache: InputCache::new(),
            };

            Vg {
                real: state.clone(),
                prediction: state,
                tick: Tick::Real,
                options: Default::default(),
                network: Network::new().await.unwrap(),
                event_queue: vec![],
                local_player: PlayerId(0),
                players: vec![],
                assets: AssetLoader::new(),
                // debug_menu: Default::default(),
            }
        });

        if std::env::var("VG_DEDICATED") != Ok("1".into()) {
            vg.run_regular(rt);
        } else {
            vg.run_dedicated(rt);
        }
    }

    // run as a regular game, with graphics and all
    fn run_regular(mut self, rt: tokio::runtime::Runtime)
    where
        G: 'static,
    {
        let mut last_predict = Instant::now();

        let ev = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("VG")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&ev)
            .unwrap();

        let mut gfx = rt.block_on(gfx::Gfx::new(&window));

        ev.run(move |event, _, flow| {
            use winit::event::{
                ElementState, Event as WEvent, KeyboardInput, VirtualKeyCode, WindowEvent,
            };
            *flow = ControlFlow::Poll;

            // g.egui_platform.handle_event(&event);

            if let Some(ev) = EventKind::new(&event) {
                self.queue_event(ev);
            }

            match event {
                // All events are cleared, now we can run all the book keeping
                WEvent::MainEventsCleared => {
                    rt.block_on(async {
                        self.options.predict_rate =
                            (1.0 / last_predict.elapsed().as_secs_f32()) as usize;
                        last_predict = Instant::now();

                        window.set_title(&format!(
                            "VG {} | {:.2}pred/s | Player {}",
                            if self.network.is_host() {
                                "host"
                            } else {
                                "client"
                            },
                            self.options.predict_rate,
                            self.local_player,
                        ));

                        // if self.debug_menu.state.is_none() {
                        if self.update_network().await.unwrap() {
                            self.run_real_tick();
                        } else {
                            self.run_prediction_tick();
                        }
                        // } else {
                        // self.tick = Tick::Real;
                        // self.run_prediction_tick();
                        // }

                        // do render
                        gfx.draw(&mut self).await.unwrap();
                    });
                }
                WEvent::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Tab),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    // self.debug_menu.visible = !self.debug_menu.visible;
                }
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

    // run as a dedicated server, so don't do any prediction ticks
    fn run_dedicated(mut self, rt: tokio::runtime::Runtime) {
        rt.block_on(async {
            loop {
                if self.update_network().await.unwrap() {
                    self.run_real_tick();
                }
            }
        });
    }

    fn queue_event(&mut self, kind: EventKind) {
        self.event_queue.push(kind.clone());
        self.prediction.events.push(Event {
            player: self.local_player,
            kind,
        });
    }

    // run actual game logic
    fn run_real_tick(&mut self) {
        self.tick = Tick::Real;

        self.real.input_cache.tick();
        for ev in self.events() {
            self.real.input_cache.event(&ev);
        }

        Game::update(self);
    }

    // run predictive game logic
    fn run_prediction_tick(&mut self) {
        if self.tick == Tick::Real {
            self.tick = Tick::Prediction;
            // Copy real state so we can do prediction
            self.prediction = self.real.clone();

            // skip into future by latency
            // this is a remedy to problems like having to lead your aim in FPS's
            let old = self.options.predict_rate;
            self.options.predict_rate = (1.0 / self.latency().as_secs_f32()) as usize;
            Game::update(self);
            self.options.predict_rate = old;
        }

        self.prediction.input_cache.tick();
        for ev in self.events() {
            self.prediction.input_cache.event(&ev);
        }

        Game::update(self);
    }

    // prediction/reality accessors
    fn universe(&self) -> &Universe<G> {
        match self.tick {
            Tick::Real => &self.real,
            Tick::Prediction => &self.prediction,
        }
    }

    fn universe_mut(&mut self) -> &mut Universe<G> {
        match self.tick {
            Tick::Real => &mut self.real,
            Tick::Prediction => &mut self.prediction,
        }
    }

    /* public API */

    /// Get all the events for this update
    pub fn events(&self) -> Vec<Event> {
        self.universe().events.clone()
    }

    /// Get the latest state of a key
    pub fn key(&self, player: PlayerId, key: event::Key) -> event::Digital {
        self.input().key(player, key)
    }

    /// Get the cached state of all inputs
    pub fn input(&self) -> &InputCache {
        &self.universe().input_cache
    }

    /// Access connected players through a player slot
    ///
    /// Think of this of how multiple controllers behave on consoles; New controllers get added to the end, but
    /// removing a controller does not shift other controllers down, instead leaving an empty position that might
    /// get filled later
    pub fn player_id(&self, slot: usize) -> Option<PlayerId> {
        self.players.get(slot).copied().flatten()
    }

    /// Get the player slot based on a player Id, the inverse of [player_id]. None if the player is not connected
    pub fn player_slot(&self, id: PlayerId) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find_map(|(i, x)| if *x == Some(id) { Some(i) } else { None })
    }

    /// Get all currently connected players
    pub fn players(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .copied()
            .filter_map(std::convert::identity)
            .collect()
    }

    /// Get current game tick
    ///
    /// Note: This is not representative of actual game time.
    /// [runtime] will not be equal to [tick] / [tick_rate]
    // pub fn tick(&self) -> usize {
    //     self.tick
    // }

    /// Get current tick rate
    pub fn tick_rate(&self) -> usize {
        match self.tick {
            Tick::Real => self.options.tick_rate,
            Tick::Prediction => self.options.predict_rate,
        }
    }

    /// Get the time elapsed since last update
    pub fn delta_time(&self) -> Duration {
        Duration::from_secs(1) / self.tick_rate() as u32
    }

    // TODO: Make this apply only after current tick
    pub fn set_tick_rate(&mut self, rate: usize) {
        self.options.tick_rate = rate;
    }

    /// Set remote server host
    pub fn set_remote(&mut self, _host: Option<&str>) {
        // self.network.set_remote(host);
    }
}

impl<G: Game> std::ops::Deref for Vg<G> {
    type Target = G;
    fn deref(&self) -> &Self::Target {
        &self.universe().state
    }
}

impl<G: Game> std::ops::DerefMut for Vg<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.universe_mut().state
    }
}
