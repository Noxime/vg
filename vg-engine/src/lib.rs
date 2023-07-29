use std::sync::Arc;

use head::Head;
use vg_asset::{Assets, Asset};
use vg_runtime::executor::WasmInstance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopWindowTarget,
};

mod head;
mod runtime;

#[cfg(target_os = "android")]
pub mod platform {
    mod android;
    pub use android::*;
}

#[cfg(not(target_os = "android"))]
pub mod platform {
    mod desktop;
    pub use desktop::*;
}

pub struct Engine {
    config: EngineConfig,
    head: Option<Head>,
    /// Is engine instance alive? False if should exit
    alive: bool,
    /// Is the app lifetime between Resume and Suspended events
    between_resumes: bool,
    /// Asset server
    assets: Arc<Assets>,
    /// Game logic instance
    instance: Asset<WasmInstance>,
}

#[derive(Clone)]
pub struct EngineConfig {
    /// Run in headless mode
    pub headless: bool,
    /// File system path to game binary
    pub path: String,
}

impl EngineConfig {
    pub fn new() -> EngineConfig {
        EngineConfig {
            headless: false,
            path: String::from("target/wasm32-wasi/debug/my-game.wasm"),
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
    pub fn event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ensure_window(target);

        match event {
            Event::Resumed => self.between_resumes = true,
            Event::Suspended => self.between_resumes = false,
            Event::WindowEvent { window_id, event } if self.is_my_window(window_id) => {
                match event {
                    WindowEvent::Resized(size) => {
                        self.resize(*size);
                    }
                    WindowEvent::CloseRequested => {
                        self.alive = false;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if self.is_my_window(window_id) => {
                self.render();
            }
            Event::MainEventsCleared => {
                self.run_frame();

                self.redraw();
            }
            _ => (),
        }
    }

    /// Is this engine instance still active
    ///
    /// If this returns false, game should exit
    pub fn alive(&self) -> bool {
        self.alive
    }
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
