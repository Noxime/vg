#[cfg(feature = "debug")]
mod debug;
mod gfx;
pub mod runtime;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use gfx::Gfx;
use log::{debug, trace, warn};
use runtime::Runtime;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    window: Arc<Window>,
    gfx: Gfx,
    start_time: Instant,
    #[cfg(feature = "debug")]
    debug: debug::DebugData,
}

impl Engine {
    pub fn run<RT: Runtime + 'static, F: Fn() -> bool + 'static>(code: &[u8], idle_task: F) {
        if let Err(err) = emoji_logger::try_init() {
            warn!("Failed to set logger: {}", err);
        }

        let mut runtime = RT::load(&code).expect("Loading the runtime failed");

        let events = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("vg-main")
                .build(&events)
                .expect("Failed to initialize a window"),
        );

        let mut engine = Engine {
            #[cfg(feature = "debug")]
            debug: debug::DebugData::new(window.clone()),
            gfx: pollster::block_on(Gfx::new(window.clone())),
            window,
            start_time: Instant::now(),
        };

        events.run(move |ev, _, flow| {
            *flow = ControlFlow::Poll;

            // hosting process has decided it is time for us to die
            if !idle_task() {
                debug!("Idle task check asked to close");
                *flow = ControlFlow::Exit;
                return;
            }

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
                    runtime.run_tick(&mut engine).unwrap();
                }
                _ => (),
            }
        })
    }

    fn call(&mut self, call: vg_types::Call) {
        match call {
            vg_types::Call::Present => {
                let runtime = self.start_time.elapsed();

                #[cfg(feature = "debug")]
                self.debug.platform.update_time(runtime.as_secs_f64());

                self.gfx.present(
                    #[cfg(feature = "debug")]
                    &mut self.debug,
                );
            }
            call => trace!("Call: {:#?}", call),
        }
    }
}
