#![feature(
    arbitrary_self_types,
    duration_saturating_ops,
    async_closure,
    never_type,
    variant_count,
    // associated_type_defaults,
)]

use log::*;
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::{Duration, Instant};

mod network;
use network::Network;
mod event;
pub use event::{Event, EventKind, PlayerId};
// mod serde_hash; TODO
mod gfx;
pub use gfx::Model;
pub mod assets;

pub use ultraviolet::transform::Similarity3 as Transform;

pub trait Game: Sized + Serialize + DeserializeOwned {
    fn update(self: &mut Vg<Self>);
}

fn dupe_state<G: Game>(state: &G) -> G {
    let start = Instant::now();
    let state = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    trace!("Game state duplication took {:.2?}", start.elapsed());
    state
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
    state: G,
    prediction_state: G,
    // are we real or prediction tick
    tick: Tick,
    options: Options,
    network: Network,
    // events queued up for next tick
    event_queue: Vec<EventKind>,
    // events for this tick
    events: Vec<Event>,
    // events that are used during prediction ticks
    predict_events: Vec<Event>,
    my_id: PlayerId,
    debug_menu: gfx::ui::Ui,
}

impl<G: Game> Vg<G> {
    pub fn run(state: G)
    where
        G: 'static,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_name("vg-runtime")
            .build()
            .unwrap();

        let vg = rt.block_on(async {
            Vg {
                prediction_state: dupe_state(&state),
                state,
                tick: Tick::Real,
                options: Default::default(),
                network: Network::new().await.unwrap(),
                event_queue: vec![],
                predict_events: vec![],
                events: vec![],
                my_id: PlayerId(0),
                debug_menu: Default::default(),
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
        // let window = WindowBuilder::new()
        //     .with_title("VG")
        //     .with_inner_size(winit::dpi::PhysicalSize::new(480, 360))
        //     .build(&ev)
        //     .unwrap();

        let mut g = rt.block_on(gfx::Gfx::new(&ev));

        ev.run(move |event, _, flow| {
            use winit::event::{
                ElementState, Event as WEvent, KeyboardInput, VirtualKeyCode, WindowEvent,
            };
            *flow = ControlFlow::Poll;

            g.egui_platform.handle_event(&event);

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

                        if self.debug_menu.state.is_none() {
                            if self.update_network().await.unwrap() {
                                self.run_real_tick();
                            } else {
                                self.run_prediction_tick();
                            }
                        } else {
                            self.tick = Tick::Real;
                            self.run_prediction_tick();
                        }

                        // do render
                        g.present(
                            &mut self,
                            gfx::Graph::new(),
                            gfx::Camera::new(Default::default(), Default::default(), 90.0),
                        )
                        .await;
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
                    self.debug_menu.visible = !self.debug_menu.visible;
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
        self.predict_events.push(Event {
            player: self.i_am(),
            kind,
        });
    }

    // run actual game logic
    fn run_real_tick(&mut self) {
        self.tick = Tick::Real;
        Game::update(self);
    }

    // run predictive game logic
    fn run_prediction_tick(&mut self) {
        if self.tick == Tick::Real {
            self.tick = Tick::Prediction;
            // Copy real state so we can do prediction
            self.prediction_state = dupe_state(&self.state);
            self.predict_events = vec![];

            // skip into future by latency
            // this is a remedy to problems like having to lead your aim in FPS's
            let old = self.options.predict_rate;
            self.options.predict_rate = (1.0 / self.latency().as_secs_f32()) as usize;
            Game::update(self);
            self.options.predict_rate = old;
        }
        Game::update(self);
    }

    /* public API */

    /// Get all the events for this update
    pub fn events(&self) -> Vec<Event> {
        match self.tick {
            Tick::Real => self.events.clone(),
            Tick::Prediction => self.predict_events.clone(),
        }
    }

    /// Get the local player ID
    pub fn i_am(&self) -> PlayerId {
        self.my_id
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

impl<S> std::ops::Deref for Vg<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        match self.tick {
            Tick::Real => &self.state,
            Tick::Prediction => &self.prediction_state,
        }
    }
}

impl<S> std::ops::DerefMut for Vg<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.tick {
            Tick::Real => &mut self.state,
            Tick::Prediction => &mut self.prediction_state,
        }
    }
}
