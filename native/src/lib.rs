mod assets;
#[cfg(feature = "debug")]
mod debug;
mod gfx;
pub mod runtime;
mod sfx;
mod util;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use assets::Assets;
use futures::future::join_all;
use gfx::Gfx;
use runtime::Runtime;
use sfx::Sfx;
use tokio::runtime::Runtime as Tokio;
use tracing::{debug, info, trace, warn};
use tracing_subscriber::prelude::*;
use vg_types::{Call, DrawCall, PlayCall};
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    window: Arc<Window>,
    gfx: Gfx,
    sfx: Sfx,
    start_time: Instant,
    assets: Assets,
    #[cfg(feature = "debug")]
    debug: debug::DebugData,
    presented: bool,
}

impl Engine {
    pub fn run<RT, F>(mut idle_task: F) -> !
    where
        RT: Runtime + 'static,
        F: FnMut() -> Option<Vec<u8>> + 'static,
    {
        let tokio = tokio::runtime::Builder::new_multi_thread()
            .thread_name("vg-worker")
            .enable_all()
            .build()
            .unwrap();

        // console_error_panic_hook::set_once();
        // tracing_wasm::set_as_global_default();

        let mut runtime = None;
        let mut tick_runtime = None;

        let events = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("vg-main")
                .build(&events)
                .expect("Failed to initialize a window"),
        );

        // #[cfg(target_arch = "wasm32")]
        // {
        //     use winit::platform::web::WindowExtWebSys;

        //     let canvas = window.canvas();

        //     let window = web_sys::window().unwrap();
        //     let document = window.document().unwrap();
        //     let body = document.body().unwrap();

        //     body.append_child(&canvas)
        //         .expect("Append canvas to HTML body");
        // }

        #[cfg(feature = "debug")]
        let debug = debug::DebugData::new(window.clone());

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let mut engine = Engine {
            #[cfg(feature = "debug")]
            debug,
            gfx: tokio.block_on(Gfx::new(window.clone())),
            sfx: Sfx::new(),
            assets: Assets::new(),
            window,
            start_time: Instant::now(),
            presented: false,
        };

        let time_tick = Duration::from_millis(10);
        let mut next_tick = Instant::now();
        let mut last_frame = Instant::now();
        let mut shown_tick = false;

        events.run(move |ev, _, flow| {
            *flow = ControlFlow::Poll;

            // hosting process has decided it is time for us to die
            if let Some(code) = idle_task() {
                debug!("Idle task reloaded code");
                tick_runtime = Some(RT::load(&code).expect("Loading the runtime failed"));
                runtime = None;
                return;
            }

            let tick_runtime = if let Some(ref mut rt) = tick_runtime {
                rt
            } else {
                return;
            };

            #[cfg(feature = "debug")]
            {
                engine.debug.platform.handle_event(&ev);
                engine.debug.tick_time = time_tick;
            }

            match ev {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *flow = ControlFlow::Exit,
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    engine.gfx.resize(size);
                }
                #[cfg(feature = "debug")]
                Event::WindowEvent {
                    event: WindowEvent::ReceivedCharacter('§'),
                    ..
                } => {
                    debug!("Toggled debug UI visibility");
                    engine.debug.visible = !engine.debug.visible;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input,
                            is_synthetic: false,
                            ..
                        },
                    ..
                } => {
                    if let Some(key) = input.virtual_keycode.and_then(util::winit_to_key) {
                        match input.state {
                            winit::event::ElementState::Pressed => {
                                tick_runtime.send(vg_types::Response::Down(key))
                            }
                            winit::event::ElementState::Released => {
                                tick_runtime.send(vg_types::Response::Up(key))
                            }
                        }
                    }
                }
                // all events for an update handled
                Event::MainEventsCleared => {
                    // we should run fixed tick
                    if next_tick < Instant::now() && shown_tick {
                        trace!("Tick");
                        next_tick += time_tick;
                        runtime = None;

                        // engine.run_till_present(tick_runtime);
                        tokio.block_on(engine.run_till_present(tick_runtime));

                        // Adjust the time by one tick. This is determenistic
                        tick_runtime.send(vg_types::Response::Time(time_tick.as_secs_f64()));

                        #[cfg(feature = "debug")]
                        {
                            shown_tick = !engine.debug.force_smooth;
                        }
                    } else {
                        // still waiting for fixed tick, so draw render ticks
                        shown_tick = true;

                        let frame_runtime = if let Some(ref mut rt) = runtime {
                            rt
                        } else {
                            runtime = Some(tick_runtime.duplicate().unwrap());
                            runtime.as_mut().unwrap()
                        };

                        tokio.block_on(engine.run_till_present(frame_runtime));

                        // Pass a frames worth of time. Non-determenistic, but its okay because we rollback each tick
                        let elapsed = last_frame.elapsed();
                        frame_runtime.send(vg_types::Response::Time(elapsed.as_secs_f64()));
                        last_frame += elapsed;
                    }
                }
                _ => (),
            }
        })
    }

    async fn run_till_present<RT: Runtime>(&mut self, rt: &mut RT) {
        puffin::profile_function!();

        let mut calls = vec![];
        let mut draws = vec![];
        let mut plays = vec![];

        let mut presented = false;
        while !presented {
            for call in rt.run_tick().unwrap().drain(..) {
                // stop ticking once we complete a frame
                if matches!(call, Call::Present) {
                    presented = true;
                }

                // split calls into different categories so we can do concurrency
                match call {
                    Call::Draw(call) => draws.push(call),
                    Call::Play(call) => plays.push(call),
                    call => calls.push(call),
                }
            }
        }

        let assets = &self.assets;

        // Turn our asset, trans pairs into loading async tasks
        let mut draw_tasks = vec![];
        for DrawCall { asset, trans } in draws {
            draw_tasks.push(async move { (assets.get(&asset).await, trans) });
        }

        let mut play_tasks = vec![];
        for PlayCall { asset } in plays {
            play_tasks.push(async move { assets.get(&asset).await });
        }

        let (draws, plays) = futures::join!(join_all(draw_tasks), join_all(play_tasks));

        for (asset, trans) in draws {
            self.gfx.draw_sprite(asset, trans).await;
        }

        for asset in plays {
            self.sfx.play_sound(asset).await;
        }

        for call in calls {
            match call {
                Call::Present => {
                    self.presented = true;
                    let runtime = self.start_time.elapsed();

                    #[cfg(feature = "debug")]
                    self.debug.platform.update_time(runtime.as_secs_f64());

                    self.gfx
                        .present(
                            #[cfg(feature = "debug")]
                            &mut self.debug,
                        )
                        .await;
                }
                Call::Print(msg) => {
                    info!("{}", msg);
                    #[cfg(feature = "debug")]
                    self.debug.print(msg);
                }
                Call::Exit => {
                    panic!()
                }
                Call::Play(..) | Call::Draw(..) => unreachable!(),
            }
        }
    }
}
