#![feature(try_trait_v2)]
#![feature(array_windows)]
#![allow(non_local_definitions)]

use runtime::WorldState;
use vg_asset::{Asset, Assets};
use vg_runtime::executor::WasmInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopWindowTarget,
};

mod check;
mod head;
mod platform;
mod prelude;
mod runtime;

pub(crate) use prelude::*;

use head::Head;

pub use runtime::{RuntimeInstant, SaveState};

pub struct Engine {
    config: EngineConfig,
    /// If not headless, everything related to engine visuals. Windowing/gfx
    head: Option<Head>,
    /// Is engine instance alive? False if should exit
    alive: bool,
    /// Is the app lifetime between Resume and Suspended events
    between_resumes: bool,
    /// Asset server
    assets: Arc<Assets>,
    /// Game logic instance
    instance: Asset<WasmInstance>,
    /// Current engine time
    instant: RuntimeInstant,
    /// Most recently calculate world state
    world: WorldState,
}

#[derive(Clone)]
pub struct EngineConfig {
    /// Run in headless mode
    pub headless: bool,
    /// File system path to game binary
    pub path: String,
    /// URL of the signaling service. Unused if not connected to a room
    pub signaling: String,
    /// Room to connect to, if networking is used
    pub room: Option<String>,
}

impl EngineConfig {
    pub fn new() -> EngineConfig {
        EngineConfig {
            headless: false,
            // backends: wgpu::Backends::all(),
            path: "target/wasm32-wasi/debug/my-game.wasm".into(),
            signaling: "ws://vg.noxim.xyz:3536/".into(),
            room: None,
        }
    }
}

/// Some platforms don't have proper Resumed/Suspended lifecycles. Important for
/// when using an external event loop
fn has_app_lifecycle() -> bool {
    ["android", "ios"].contains(&std::env::consts::OS)
}

impl Engine {
    /// Create a new engine instance with default configuration
    pub fn new() -> Engine {
        Engine::with_config(EngineConfig::new())
    }

    /// Create a new engine with specified configuration
    pub fn with_config(config: EngineConfig) -> Engine {
        let assets = Assets::new();
        Engine {
            head: None,
            alive: true,
            between_resumes: !has_app_lifecycle(),
            instance: assets.get(&config.path),
            instant: RuntimeInstant::EPOCH,
            world: Default::default(),
            assets,
            config,
        }
    }

    /// Access the engine configuration. Changes are not guaranteed to apply in
    /// real time
    pub fn config_mut(&mut self) -> &mut EngineConfig {
        &mut self.config
    }

    /// Access the asset server
    pub fn assets(&self) -> &Arc<Assets> {
        &self.assets
    }

    /// Process a winit event
    #[profile]
    pub fn event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) -> Nil {
        self.ensure_window(target);

        // Filter out events that arent belong to us
        self.check_my_event(event)?;

        match event {
            Event::Resumed => self.between_resumes = true,
            Event::Suspended => self.between_resumes = false,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    self.resize(*size);
                }
                WindowEvent::CloseRequested => {
                    self.alive = false;
                }
                WindowEvent::RedrawRequested => {
                    self.render();
                    profiling::finish_frame!();
                }
                _ => (),
            },
            Event::AboutToWait => {
                self.redraw();

                // TODO: Winit bug, Redraws are not always delivered. For 
                // example, when out of focus on windows
                self.render();
            }
            _ => (),
        }

        Nil
    }

    /// Keep calling this function as often as possible
    #[profile]
    pub fn poll(&mut self) -> PollResult {
        check::check_default(|| {
            // TODO: Tick rate
            self.run_tick()?;

            Check::Pass(PollResult::Tick {})
        })
    }

    /// Checks if the event is relevant to our engine instance
    fn check_my_event(&self, event: &Event<()>) -> Check {
        // These are not specific to any instance, always accept
        if matches!(
            event,
            Event::AboutToWait | Event::Resumed | Event::Suspended
        ) {
            return PASS;
        }

        // These are per window
        let Event::WindowEvent { window_id, .. } = event else {
            return FAIL;
        };
        self.is_my_window(*window_id)
    }

    /// Is this engine instance still active
    ///
    /// If this returns false, game should exit
    pub fn alive(&self) -> bool {
        self.alive
    }

    /// Runs async close blockingly
    fn block_on<T>(&self, f: impl std::future::Future<Output = T>) -> T {
        // TODO: Tokio
        pollster::block_on(f)
    }
}

/// What happened after a call to `Engine::poll`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PollResult {
    /// Nothing of mention happened
    #[default]
    None,
    /// A tick has occurred
    Tick,
}

/*
async fn run_host(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut host = HostData::new();
    let mut instant = Instant::now();

    loop {
        tokio::task::yield_now().await;

        host.poll(&mut socket)?;

        // It is time for a server tick
        let elapsed = instant.elapsed();
        if elapsed >= Duration::from_millis(1000) {
            // Execute one tick
            loop {
                match instance.step() {
                    WaitReason::Startup => continue,
                    WaitReason::Present => break,
                }
            }

            // Announce new tick
            let data = instance.get_data();
            host.tick(&mut socket, &data, elapsed)?;

            instant += elapsed;
        }
    }
}

async fn run_client(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut client = ClientData::new();

    loop {
        tokio::task::yield_now().await;

        let Some(confirm) = client.poll(&mut socket)? else { continue };

        // Deserialize state if server pushed
        if let Some(data) = confirm.state()? {
            instance.set_data(&data);
        }

        // Execute one tick
        loop {
            match instance.step() {
                WaitReason::Startup => continue,
                WaitReason::Present => break,
            }
        }

        let data = instance.get_data();
        // Desync
        if confirm.diverged(&data) {
            client.desync(&mut socket)?;
        }
    }
}

*/
