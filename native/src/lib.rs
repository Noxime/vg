mod assets;

mod debug;
mod gfx;
pub mod runtime;
mod sfx;
mod util;
mod net;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use assets::Assets;
use debug::DebugUi;
use futures::future::join_all;
use gfx::Gfx;
use glam::UVec2;
use runtime::Runtime;
use sfx::Sfx;
use tracing::{debug, info, trace};
use tracing_subscriber::prelude::*;
use vg_types::{Call, DrawCall, PlayCall};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    #[allow(unused)]
    window: Arc<Window>,
    gfx: Gfx,
    sfx: Sfx,
    start_time: Instant,
    assets: Assets,
    debug: DebugUi,
}

impl Engine {
    pub fn run<RT, F>(mut idle_task: F) -> !
    where
        RT: Runtime + 'static,
        F: FnMut() -> Option<Vec<u8>> + 'static,
    {
        let sfx = Sfx::new();

        let tokio = tokio::runtime::Builder::new_multi_thread()
            .thread_name("vg")
            .enable_all()
            .build()
            .unwrap();

        // console_error_panic_hook::set_once();
        // tracing_wasm::set_as_global_default();

        let mut runtime = None;
        let mut runtime_smooth = None;

        let events = EventLoop::new();
        let mut builder = WindowBuilder::new().with_title("vg-main");

        #[cfg(target_os = "windows")]
        {
            // Disable drag and drop because of windows COM stuff, idk
            use winit::platform::windows::WindowBuilderExtWindows;
            builder = builder.with_drag_and_drop(false);
        }

        let window = Arc::new(
            builder
                .build(&events)
                .expect("Failed to initialize a window"),
        );

        
        let debug = DebugUi::new(window.clone(), RT::NAME);

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let mut engine = Engine {
            
            debug,
            sfx,
            gfx: tokio.block_on(Gfx::new(window.clone())),
            assets: Assets::new(),
            window,
            start_time: Instant::now(),
        };

        let tickrate = Duration::from_millis(100);
        let mut next_tick = Instant::now();
        let mut smooth_frame = Instant::now();
        let mut smoothed_frames = 0;

        events.run(move |ev, _, flow| {
            puffin::profile_scope!("main");
            *flow = ControlFlow::Poll;

            // hosting process has decided it is time for us to die
            if let Some(code) = idle_task() {
                debug!("Idle task reloaded code");
                runtime = Some(RT::load(&code).expect("Loading the runtime failed"));
                return;
            }

            // Don't run when we have nothing... to run
            let runtime = match runtime {
                Some(ref mut rt) => rt,
                None => return,
            };

            
            {
                engine.debug.platform.handle_event(&ev);
                engine.debug.tick_time = tickrate;
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
                    engine.gfx.resize(UVec2::new(size.width, size.height));
                }
                
                Event::WindowEvent {
                    event: WindowEvent::ReceivedCharacter('/'),
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
                                runtime.send(vg_types::Response::Down(key))
                            }
                            winit::event::ElementState::Released => {
                                runtime.send(vg_types::Response::Up(key))
                            }
                        }
                    }
                }
                // all events for an update handled
                Event::MainEventsCleared => {
                    // we should run fixed tick
                    if next_tick <= Instant::now() {
                        trace!("Tick");
                        next_tick += tickrate;

                        // Run a proper tick
                        tokio.block_on(engine.run_till_present(runtime, tickrate));

                        // The smoothed runtime is invalid state now
                        runtime_smooth = None;
                        smooth_frame = Instant::now();
                        engine.debug.smoothed_frames = smoothed_frames;
                        smoothed_frames = 0;
                    } else {
                        trace!("Smooth");
                        let runtime_smooth = runtime_smooth.get_or_insert_with(|| runtime.duplicate().unwrap());

                        let elapsed = smooth_frame.elapsed();
                        smooth_frame += elapsed;
                        smoothed_frames += 1;
                        tokio.block_on(engine.run_till_present(runtime_smooth, elapsed));

                        // // Pass a frames worth of time. Non-determenistic, but its okay because we rollback each tick
                        // let elapsed = smooth_frame.elapsed();
                        // smooth_frame += elapsed;
                    }
                }
                _ => (),
            }
        })
    }

    async fn run_till_present<RT: Runtime>(&mut self, rt: &mut RT, dt: Duration) {
        puffin::profile_function!();

        let mut calls = vec![];
        let mut draws = vec![];
        let mut plays = vec![];

        for call in rt.run_tick(dt).unwrap().drain(..) {
            // split calls into different categories so we can do concurrency
            match call {
                Call::Draw(call) => draws.push(call),
                Call::Play(call) => plays.push(call),
                call => calls.push(call),
            }
        }

        let assets = &self.assets;

        // Turn our asset, trans pairs into loading async tasks
        let mut draw_tasks = vec![];
        for DrawCall { asset, trans } in draws {
            draw_tasks.push(async move { (assets.get(&asset).await, trans) });
        }

        let mut play_tasks = vec![];
        for PlayCall { asset, looping } in plays {
            play_tasks.push(async move { (assets.get(&asset).await, looping) });
        }

        let (draws, plays) = futures::join!(join_all(draw_tasks), join_all(play_tasks));

        for (asset, trans) in draws {
            self.gfx.draw_sprite(asset, trans).await;
        }

        for (asset, looping) in plays {
            self.sfx.play_sound(asset, looping).await;
        }

        for call in calls {
            match call {
                Call::Print(msg) => {
                    info!("{}", msg);
                    
                    self.debug.print(msg);
                }
                Call::Exit => {
                    panic!()
                }
                Call::Play(..) | Call::Draw(..) => unreachable!(),
            }
        }

        let runtime = self.start_time.elapsed();

        
        self.debug.platform.update_time(runtime.as_secs_f64());

        self.gfx
            .present(
                
                &mut self.debug,
            )
            .await;
    }
}
