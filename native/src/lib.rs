mod assets;
#[cfg(feature = "debug")]
mod debug;
mod gfx;
pub mod runtime;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use assets::Assets;
use gfx::Gfx;
use runtime::Runtime;
use tracing::{debug, info, trace};
use tracing_subscriber::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    window: Arc<Window>,
    gfx: Gfx,
    start_time: Instant,
    assets: Assets,
    #[cfg(feature = "debug")]
    debug: debug::DebugData,
    presented: bool,
    mask: bool,
}

impl Engine {
    pub fn run<RT, F>(mut idle_task: F) -> !
    where
        RT: Runtime + 'static,
        F: FnMut() -> Option<Vec<u8>> + 'static,
    {
        let mut runtime = None;
        let mut tick_runtime = None;

        let events = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("vg-main")
                .build(&events)
                .expect("Failed to initialize a window"),
        );

        #[cfg(feature = "debug")]
        let debug = debug::DebugData::new(window.clone());

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let mut engine = Engine {
            #[cfg(feature = "debug")]
            debug,
            gfx: pollster::block_on(Gfx::new(window.clone())),
            assets: Assets::new(),
            window,
            start_time: Instant::now(),
            presented: false,
            mask: false,
        };

        let time_tick = Duration::from_millis(500);
        let mut next_tick = Instant::now();
        let mut last_frame = Instant::now();

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

            let frame_runtime = if let Some(ref mut rt) = runtime {
                rt
            } else {
                runtime = Some(tick_runtime.duplicate().unwrap());
                runtime.as_mut().unwrap()
            };

            #[cfg(feature = "debug")]
            engine.debug.platform.handle_event(&ev);

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
                // all events for an update handled
                Event::MainEventsCleared => {
                    engine.presented = false;

                    if next_tick < Instant::now() {
                        trace!("Tick");
                        next_tick += time_tick;
                        runtime = None;

                        engine.mask = true;
                        while !engine.presented {
                            tick_runtime.run_tick(&mut engine).unwrap();
                        }
                        tick_runtime.send(vg_types::Response::Time(time_tick.as_secs_f64()));
                    } else {
                        engine.mask = false;
                        while !engine.presented {
                            frame_runtime.run_tick(&mut engine).unwrap();
                        }
                        let elapsed = last_frame.elapsed();
                        frame_runtime
                            .send(vg_types::Response::Time(elapsed.as_secs_f64()));
                        last_frame += elapsed;
                    }
                }
                _ => (),
            }
        })
    }

    fn call(&mut self, call: vg_types::Call) {
        puffin::profile_function!();

        match call {
            vg_types::Call::Present => {
                self.presented = true;
                if self.mask {
                    return
                }

                let runtime = self.start_time.elapsed();

                #[cfg(feature = "debug")]
                self.debug.platform.update_time(runtime.as_secs_f64());

                self.gfx.present(
                    #[cfg(feature = "debug")]
                    &mut self.debug,
                );

            }
            vg_types::Call::Draw { asset, trans } => {
                if self.mask {
                    return
                }

                let img = self.assets.load(&asset);
                self.gfx.draw_sprite(img, trans);
            }
            vg_types::Call::Print(msg) => {
                info!("{}", msg);

                #[cfg(feature = "debug")]
                self.debug.print(msg);
            }
            call => trace!("Call: {:#?}", call),
        }
    }
}
