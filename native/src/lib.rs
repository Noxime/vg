mod assets;
mod debug;
mod gfx;
mod ids;
mod net;
pub mod runtime;
mod sfx;
mod util;
use assets::Assets;
use debug::DebugUi;
use futures::future::join_all;
use gfx::Gfx;
use runtime::Runtime;
use sfx::Sfx;
use std::net::Ipv4Addr;
use std::sync::mpsc::{self as sync_mpsc};
use std::{sync::Arc, time::Instant};
use tokio::sync::mpsc::{self, Sender};
use tracing::{debug, info};
use tracing_subscriber::prelude::*;
use vg_types::Event;
use vg_types::{Call, DrawCall, PlayCall};
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    #[allow(unused)]
    window: Arc<Window>,
    gfx: Gfx,
    sfx: Sfx,
    assets: Assets,
    debug: DebugUi,
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("vg")
        .enable_all()
        .build()
        .unwrap()
}

async fn handle(ev: WinitEvent<'_, ()>, event_tx: Sender<Event>) -> anyhow::Result<()> {
    match ev {
        WinitEvent::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            std::process::exit(0);
        }
        WinitEvent::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            if let Some(key) = input.virtual_keycode.and_then(util::winit_to_key) {
                match input.state {
                    winit::event::ElementState::Pressed => {
                        event_tx.send(vg_types::Event::Down(key)).await?
                    }
                    winit::event::ElementState::Released => {
                        event_tx.send(vg_types::Event::Up(key)).await?
                    }
                }
            }
        }
        WinitEvent::MainEventsCleared => {}
        _ => (),
    }

    Ok(())
}

impl Engine {
    pub fn run<RT, F>(mut load_task: F) -> !
    where
        RT: Runtime + 'static,
        F: FnMut() -> Option<Vec<u8>> + 'static,
    {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let code = load_task().unwrap();
        let tokio = runtime();

        // Start the dedicated server
        if true {
            let runtime = RT::load(&code).unwrap();
            tokio.spawn(async {
                net::server(runtime).await.expect("Failed to run server");
                info!("Server exit");
            });
        }

        // Network client
        let (event_tx, event_rx) = mpsc::channel(16);
        let (rt_tx, rt_rx) = sync_mpsc::channel();
        tokio.spawn(async {
            net::client::<RT>(Ipv4Addr::new(127, 0, 0, 1).into(), event_rx, rt_tx)
                .await
                .unwrap();
            info!("Client exit");
        });

        // Windowing
        let events = EventLoop::new();
        #[allow(unused_mut)]
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

        let mut engine = Engine {
            gfx: tokio.block_on(Gfx::new(window.clone())),
            debug: DebugUi::new(window.clone(), RT::NAME),
            sfx: Sfx::new(),
            assets: Assets::new(),
            window,
        };

        let mut runtime = rt_rx.recv().unwrap();
        let mut frame_time = Instant::now();

        events.run(move |ev, _, flow| {
            *flow = ControlFlow::Poll;

            // Client received updated tick
            if let Ok(rt) = rt_rx.try_recv() {
                runtime = rt;
                debug!("Client tick received");
            }

            tokio
                .block_on(handle(ev, event_tx.clone()))
                .expect("Event handler failed");

            let elapsed = frame_time.elapsed();
            let calls = runtime.run_tick(elapsed).unwrap();
            frame_time += elapsed;

            tokio.block_on(engine.process_calls(calls));
        })
    }

    async fn process_calls(&mut self, all_calls: Vec<Call>) {
        puffin::profile_function!();

        let mut calls = vec![];
        let mut draws = vec![];
        let mut plays = vec![];

        for call in all_calls {
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

        self.gfx.present(&mut self.debug).await;
    }
}
